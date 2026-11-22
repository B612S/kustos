// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Window parameters follow Luma: a 960x640 window with a 640x480 minimum,
// a GlobalDrawer for the application menu and settings as an overlay card.
Kirigami.ApplicationWindow {
    id: root

    visible: true
    width: 960
    height: 640
    minimumWidth: 640
    minimumHeight: 480
    title: qsTr("app_title")

    property var snapshots: []
    property var configuredRules: []
    property var ruleRows: []
    property var ruleSnapshots: []
    property var timelineItems: []
    property string selectedRuleId: ""
    property string selectedRuleName: ""
    property bool selectedRuleIsOther: false
    property string selectedSnapshotId: ""
    property bool snapshotsLoading: false
    property var unlockedRules: []
    property int lockGeneration: 0
    property bool hasRepo: false
    // "backup-all", "backup-rule:<id>", "edit:<id>" or "select:<id>"; used to
    // continue an action after the password dialog closed successfully.
    property string pendingAction: ""

    readonly property bool selectedRuleLocked: {
        if (root.selectedRuleIsOther)
            return root.unlockedRules.length === 0;
        if (root.selectedRuleId === "")
            return false;
        return root.unlockedRules.indexOf(root.selectedRuleId) === -1;
    }
    readonly property bool selectedRuleHasPassword: {
        if (Config.passwordMode === "common")
            return Config.commonPasswordSet;
        for (let i = 0; i < root.configuredRules.length; ++i) {
            if (root.configuredRules[i].id === root.selectedRuleId)
                return root.configuredRules[i].passwordSet;
        }
        return false;
    }

    property var backupStatus: ({
        running: false, percent: 0, state: "idle", error: "",
        speed: "", eta: "", files: 0, added: 0, snapshotId: "", current: "",
        rule: "", ruleId: ""
    })
    property var restoreStatus: ({
        running: false, percent: 0, state: "idle", error: "", current: "", mode: ""
    })

    readonly property bool busy: backupStatus.running || restoreStatus.running
    readonly property int currentProgress: restoreStatus.running ? restoreStatus.percent : backupStatus.percent
    // Context menus are plain overlay popups with a higher z than panels, so
    // they must be explicitly closed while a modal card is open.
    readonly property bool modalDialogOpen: backupDialog.opened || passwordDialog.opened
                                             || settingsDialog.opened || restoreDialog.opened
                                             || aboutDialog.opened || deleteRuleDialog.opened
                                             || deleteDialog.opened

    function dayLabel(iso) {
        const parts = String(iso).split("-");
        if (parts.length !== 3)
            return iso;
        const date = new Date(Number(parts[0]), Number(parts[1]) - 1, Number(parts[2]));
        const now = new Date();
        const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
        const diff = Math.round((today.getTime() - date.getTime()) / 86400000);
        if (diff === 0)
            return qsTr("day_today");
        if (diff === 1)
            return qsTr("day_yesterday");
        return Qt.formatDate(date, "dddd, d MMMM yyyy");
    }

    function rebuildTimeline() {
        let items = [];
        let lastDay = "";
        for (let i = 0; i < ruleSnapshots.length; ++i) {
            const snapshot = ruleSnapshots[i];
            if (snapshot.dayKey !== lastDay) {
                lastDay = snapshot.dayKey;
                items.push({ header: true, label: root.dayLabel(snapshot.dayKey) });
            }
            items.push({ header: false, snapshot: snapshot });
        }
        timelineItems = items;
    }

    function ensureSelection() {
        if (ruleSnapshots.length === 0) {
            selectedSnapshotId = "";
            return;
        }
        let found = false;
        for (let i = 0; i < ruleSnapshots.length; ++i) {
            if (ruleSnapshots[i].id === selectedSnapshotId)
                found = true;
        }
        // Do not pick the latest snapshot automatically: the browser stays
        // neutral until the user chooses one.
        if (!found)
            selectedSnapshotId = "";
    }

    function selectedSnapshot() {
        for (let i = 0; i < ruleSnapshots.length; ++i) {
            if (ruleSnapshots[i].id === selectedSnapshotId)
                return ruleSnapshots[i];
        }
        return null;
    }

    // Snapshots without a usable rule tag (created by older Kustos versions,
    // or left behind by a deleted rule) are collected in a synthetic
    // "Other backups" entry instead of disappearing from the window.
    function matchesRule(snapshot, ruleId) {
        if (ruleId === "__other__")
            return snapshot.ruleId === ""
                    || !configuredRules.some(rule => rule.id === snapshot.ruleId);
        return snapshot.ruleId === ruleId;
    }

    function latestRuleSnapshot(ruleId) {
        let latest = null;
        for (let i = 0; i < snapshots.length; ++i) {
            if (!matchesRule(snapshots[i], ruleId))
                continue;
            if (latest === null || snapshots[i].epoch > latest.epoch)
                latest = snapshots[i];
        }
        return latest;
    }

    function lastBackupLabel(snapshot) {
        if (snapshot === null)
            return "";
        const today = Qt.formatDate(new Date(), "yyyy-MM-dd");
        if (snapshot.dayKey === today)
            return snapshot.timeText;
        const yesterday = Qt.formatDate(new Date(Date.now() - 86400000), "yyyy-MM-dd");
        if (snapshot.dayKey === yesterday)
            return qsTr("day_yesterday") + " " + snapshot.timeText;
        return snapshot.dateText;
    }

    function lastBackupStale(rule, snapshot) {
        if (snapshot === null || !rule.schedule || !rule.schedule.enabled)
            return false;
        let limit = 2 * 86400;
        if (rule.schedule.mode === "interval")
            limit = Math.max(rule.schedule.minutes * 60 * 2, 2 * 3600);
        else if (rule.schedule.mode === "daily")
            limit = 36 * 3600;
        else if (rule.schedule.mode === "weekly")
            limit = 8 * 86400;
        const time = Number(snapshot.epoch);
        return time > 0 && Date.now() / 1000 - time > limit;
    }

    function countRuleSnapshots(ruleId) {
        let count = 0;
        for (let i = 0; i < snapshots.length; ++i) {
            if (matchesRule(snapshots[i], ruleId))
                count += 1;
        }
        return count;
    }

    function rebuildRuleRows() {
        configuredRules = Config.rules();
        unlockedRules = Store.status().unlockedRules;
        let rows = [];
        for (let i = 0; i < configuredRules.length; ++i) {
            const rule = configuredRules[i];
            const latest = latestRuleSnapshot(rule.id);
            rows.push(Object.assign({}, rule, {
                synthetic: false,
                locked: root.unlockedRules.indexOf(rule.id) === -1,
                snapshotCount: countRuleSnapshots(rule.id),
                lastBackupText: lastBackupLabel(latest),
                staleBackup: lastBackupStale(rule, latest)
            }));
        }

        let orphans = 0;
        for (let i = 0; i < snapshots.length; ++i) {
            if (matchesRule(snapshots[i], "__other__"))
                orphans += 1;
        }
        if (orphans > 0 && root.unlockedRules.length > 0) {
            rows.push({
                id: "__other__",
                name: qsTr("rules_other"),
                enabled: false,
                synthetic: true,
                locked: false,
                passwordSet: false,
                schedule: null,
                sources: [],
                excludes: [],
                snapshotCount: orphans,
                lastBackupText: "",
                staleBackup: false
            });
        }
        ruleRows = rows;

        let valid = false;
        for (let i = 0; i < rows.length; ++i) {
            if (rows[i].id === selectedRuleId)
                valid = true;
        }
        if (!valid) {
            let next = "";
            if (snapshots.length > 0) {
                const latestRule = snapshots[0].ruleId;
                for (let i = 0; i < rows.length; ++i) {
                    if (!rows[i].synthetic && rows[i].id === latestRule)
                        next = rows[i].id;
                }
                if (next === "" && orphans > 0)
                    next = "__other__";
            }
            if (next === "" && rows.length > 0)
                next = rows[0].id;
            selectedRuleId = next;
        }
    }

    function applyRuleFilter() {
        const readable = selectedRuleId === "__other__"
                ? unlockedRules.length > 0
                : (selectedRuleId !== "" && unlockedRules.indexOf(selectedRuleId) !== -1);
        let filtered = [];
        if (readable) {
            for (let i = 0; i < snapshots.length; ++i) {
                if (matchesRule(snapshots[i], selectedRuleId))
                    filtered.push(snapshots[i]);
            }
        }
        ruleSnapshots = filtered;

        selectedRuleIsOther = selectedRuleId === "__other__";
        selectedRuleName = "";
        for (let i = 0; i < ruleRows.length; ++i) {
            if (ruleRows[i].id === selectedRuleId) {
                selectedRuleName = ruleRows[i].name !== "" ? ruleRows[i].name
                                                           : qsTr("rule_untitled");
            }
        }

        rebuildTimeline();
        ensureSelection();
    }

    function applySnapshots() {
        snapshots = Store.snapshots();
        rebuildRuleRows();
        applyRuleFilter();
    }

    // `rustic snapshots` reads the repository index and can take a while on a
    // large repository, so it runs in the background; the timer below applies
    // the result as soon as it arrives.
    function refreshSnapshots() {
        root.rebuildRuleRows();
        if (!Store.anyUnlocked()) {
            root.snapshots = [];
            root.snapshotsLoading = false;
            root.applyRuleFilter();
            return;
        }
        Store.refreshSnapshots();
        snapshotsLoading = Store.snapshotsLoading();
        if (!snapshotsLoading)
            applySnapshots();
    }

    function pollSnapshots() {
        if (Store.snapshotsLoading())
            return;
        snapshotsLoading = false;
        applySnapshots();
        const error = Store.snapshotsError();
        if (error !== "")
            root.showPassiveNotification(qsTr("toast_load_failed").arg(error));
    }

    function ruleById(id) {
        const rules = Config.rules();
        for (let i = 0; i < rules.length; ++i) {
            if (rules[i].id === id)
                return rules[i];
        }
        return null;
    }

    function ruleDisplayName(rule) {
        return rule.name !== "" ? rule.name : qsTr("rule_untitled");
    }

    function firstEnabledRule() {
        const rules = Config.rules();
        for (let i = 0; i < rules.length; ++i) {
            if (rules[i].enabled)
                return rules[i];
        }
        return rules.length > 0 ? rules[0] : null;
    }

    // An empty rule id means "run every enabled rule"; a non-empty id runs
    // only that rule from the rules dialog. Starting a backup does not need a
    // password: the saved systemd credential is used (and a new repository is
    // initialized with a generated key). A password is asked only once for an
    // old repository that has no saved credential at all.
    function requestBackup(ruleId) {
        if (root.busy) {
            root.showPassiveNotification(qsTr("toast_backup_running"));
            return;
        }
        if (Config.rules().length === 0) {
            root.openRulesEditor();
            root.showPassiveNotification(qsTr("rules_empty"));
            return;
        }
        if (!Store.backupNeedsPassword()) {
            root.startBackup(ruleId);
            return;
        }
        const rule = ruleId === "" ? root.firstEnabledRule() : root.ruleById(ruleId);
        if (rule === null) {
            root.showPassiveNotification(qsTr("rules_empty"));
            return;
        }
        root.openPasswordForRule(rule.id, ruleId === "" ? "backup-all" : "backup-rule:" + ruleId);
    }

    function startBackup(ruleId) {
        if (root.busy) {
            root.showPassiveNotification(qsTr("toast_backup_running"));
            return;
        }
        if (Config.rules().length === 0) {
            root.openRulesEditor();
            root.showPassiveNotification(qsTr("rules_empty"));
            return;
        }
        if (ruleId !== "")
            root.selectedRuleId = ruleId;
        Backup.start(ruleId);
        root.backupStatus = Backup.poll();
        // Password resolution and repository initialization happen before the
        // worker starts; report an immediate failure instead of waiting for a
        // running -> finished transition that never happens.
        if (!root.backupStatus.running && root.backupStatus.state === "failed")
            root.showPassiveNotification(qsTr("toast_backup_failed").arg(root.backupStatus.error));
    }

    function openPasswordForRule(ruleId, action) {
        const rule = root.ruleById(ruleId);
        if (rule === null)
            return;
        root.pendingAction = action !== undefined ? action : "";
        if (Store.isRuleUnlocked(ruleId)) {
            root.runPendingAction(ruleId);
            root.refreshSnapshots();
            return;
        }
        root.selectedRuleId = ruleId;
        const name = root.ruleDisplayName(rule);

        if (Config.passwordMode === "common") {
            if (Config.commonPasswordSet)
                passwordDialog.openUnlock(ruleId, name, true);
            else
                passwordDialog.openCommon(ruleId, name, false);
            return;
        }

        if (!rule.passwordSet) {
            // The repository may already exist (a new rule or an upgrade from
            // the single-repository-key versions). A saved credential can
            // authorize the new viewing password, so ask for an existing
            // password only when there is none.
            const authRequired = Store.status().hasRepo && !Store.anyUnlocked()
                                 && !Store.canAuthorizeWithCredential();
            passwordDialog.openCreate(ruleId, name, authRequired, false);
        } else {
            passwordDialog.openUnlock(ruleId, name, false);
        }
    }

    // Opens the one-password-for-all dialog from Settings or the rule editor.
    function openCommonPassword() {
        const rule = root.firstEnabledRule();
        if (rule === null) {
            root.showPassiveNotification(qsTr("rules_empty"));
            return;
        }
        root.pendingAction = "";
        passwordDialog.openCommon(rule.id, root.ruleDisplayName(rule),
                                  Config.commonPasswordSet);
    }

    function unlockSelectedRule() {
        if (root.selectedRuleId === "__other__") {
            const rule = root.firstEnabledRule();
            if (rule !== null)
                root.openPasswordForRule(rule.id, "select:__other__");
        } else if (root.selectedRuleId !== "") {
            root.openPasswordForRule(root.selectedRuleId, "");
        }
    }

    function activateRule(id) {
        root.selectedRuleId = id;
        if (id === "__other__") {
            if (root.unlockedRules.length === 0)
                root.unlockSelectedRule();
            return;
        }
        const rule = root.ruleById(id);
        if (rule !== null && !Store.isRuleUnlocked(id))
            root.openPasswordForRule(id, "");
    }

    function runPendingAction(ruleId) {
        const action = root.pendingAction;
        root.pendingAction = "";
        root.rebuildRuleRows();
        if (action === "backup-all") {
            root.startBackup("");
        } else if (action.indexOf("backup-rule:") === 0) {
            root.startBackup(action.substring("backup-rule:".length));
        } else if (action.indexOf("edit:") === 0) {
            const id = action.substring("edit:".length);
            root.selectedRuleId = id;
            backupDialog.openRule(id);
        } else if (action.indexOf("select:") === 0) {
            root.selectedRuleId = action.substring("select:".length);
        } else if (ruleId !== "") {
            root.selectedRuleId = ruleId;
        }
    }

    function finishPassword(ruleId, mode) {
        root.runPendingAction(ruleId);
        root.refreshSnapshots();
        if (backupDialog.opened)
            backupDialog.refresh();
        const warning = Store.passwordWarning();
        if (warning !== "")
            root.showPassiveNotification(qsTr("toast_credential_failed").arg(warning));
    }

    function lockBackups() {
        if (!Store.lockNow())
            return;
        root.lockGeneration = Store.lockGeneration();
        root.onVaultLocked();
    }

    function onVaultLocked() {
        Store.clearCache();
        root.snapshots = [];
        root.snapshotsLoading = false;
        root.selectedSnapshotId = "";
        browserPanel.loadSnapshot("");
        root.rebuildRuleRows();
        root.applyRuleFilter();
        root.pendingAction = "";
        root.showPassiveNotification(qsTr("toast_locked"));
    }

    function checkLockState() {
        root.hasRepo = Store.status().hasRepo;
        const generation = Store.lockGeneration();
        if (generation !== root.lockGeneration) {
            root.lockGeneration = generation;
            root.onVaultLocked();
        }
    }


    function addRule() {
        const id = Config.addRule(qsTr("rule_new_name"));
        root.selectedRuleId = id;
        backupDialog.openRule(id);
    }

    // Opening the rule editor counts as opening the backup: a locked rule
    // asks for its viewing password first, then the editor opens after the
    // password dialog succeeds. A rule without a password can be edited
    // directly (there is nothing to unlock).
    function requestEditRule(id) {
        const rule = root.ruleById(id);
        if (rule === null)
            return;
        const hasRepo = Store.status().hasRepo;
        const locked = Config.passwordMode === "common"
                ? (Config.commonPasswordSet && hasRepo && !Store.anyUnlocked())
                : (rule.passwordSet && hasRepo && !Store.isRuleUnlocked(id));
        if (!locked) {
            backupDialog.openRule(id);
            return;
        }
        root.selectedRuleId = id;
        root.pendingAction = "edit:" + id;
        const name = root.ruleDisplayName(rule);
        if (Config.passwordMode === "common")
            passwordDialog.openUnlock(id, name, true);
        else
            passwordDialog.openUnlock(id, name, false);
    }

    // The editor now edits exactly one rule: the rule selected in the left
    // column. If there is no real rule yet, create one and open it.
    function openRulesEditor() {
        let id = selectedRuleId;
        if (id === "" || id === "__other__") {
            id = "";
            for (let i = 0; i < ruleRows.length; ++i) {
                if (!ruleRows[i].synthetic) {
                    id = ruleRows[i].id;
                    break;
                }
            }
        }
        if (id === "") {
            const rules = Config.rules();
            if (rules.length > 0)
                id = rules[0].id;
        }
        if (id === "") {
            root.addRule();
            return;
        }
        root.requestEditRule(id);
    }

    function removeSnapshot(id) {
        Store.remove(id);
        refreshSnapshots();
    }

    function statusText() {
        if (restoreStatus.running) {
            if (restoreStatus.state === "cancelling")
                return qsTr("status_cancelling");
            if (restoreStatus.mode === "open")
                return qsTr("status_opening").arg(restoreStatus.current);
            if (restoreStatus.mode === "extract")
                return qsTr("status_saving").arg(restoreStatus.current);
            return qsTr("status_restoring").arg(restoreStatus.current);
        }
        if (backupStatus.running) {
            if (backupStatus.state === "preparing")
                return qsTr("status_preparing");
            if (backupStatus.state === "cancelling")
                return qsTr("status_cancelling");
            if (backupStatus.speed === "")
                return qsTr("status_working");
            if (backupStatus.rule !== "")
                return qsTr("status_copying_rule")
                        .arg(backupStatus.rule)
                        .arg(backupStatus.speed)
                        .arg(backupStatus.eta);
            return qsTr("status_copying").arg(backupStatus.speed).arg(backupStatus.eta);
        }
        return "";
    }

    function statusSummary() {
        if (Store.repoMissing())
            return qsTr("status_repo_missing").arg(Config.repo);
        if (root.unlockedRules.length === 0)
            return qsTr("status_locked");
        return qsTr("status_summary")
                .arg(snapshots.length)
                .arg(Store.repoPath())
                .arg(App.formatBytes(Store.freeBytes()));
    }

    function handleFinishedJobs() {
        root.checkLockState();
        const nextBackup = Backup.poll();
        const previousRunning = root.backupStatus.running;
        if (previousRunning && !nextBackup.running) {
            if (nextBackup.state === "done") {
                // A backup may run while its snapshots are locked. Do not
                // select the new snapshot then: opening it would try to read
                // the repository without a password.
                if (nextBackup.ruleId !== "" && Store.isRuleUnlocked(nextBackup.ruleId)) {
                    root.selectedRuleId = nextBackup.ruleId;
                    root.selectedSnapshotId = nextBackup.snapshotId;
                    root.showPassiveNotification(qsTr("toast_backup_done"));
                } else {
                    root.showPassiveNotification(qsTr("toast_backup_done_locked"));
                }
                root.refreshSnapshots();
                if (nextBackup.error !== "")
                    root.showPassiveNotification(qsTr("toast_backup_warnings").arg(nextBackup.error));
            } else if (nextBackup.state === "failed") {
                root.showPassiveNotification(qsTr("toast_backup_failed").arg(nextBackup.error));
                // An interrupted run may still have produced a snapshot.
                root.refreshSnapshots();
            }
        }
        root.backupStatus = nextBackup;

        const nextRestore = Restore.poll();
        const restoring = root.restoreStatus.running;
        if (restoring && !nextRestore.running) {
            const saving = nextRestore.mode === "extract";
            const opening = nextRestore.mode === "open";
            if (nextRestore.state === "done") {
                if (!opening)
                    root.showPassiveNotification(saving
                        ? qsTr("toast_export_done").arg(nextRestore.added)
                        : qsTr("toast_restore_done").arg(nextRestore.added));
            } else if (nextRestore.state === "failed") {
                root.showPassiveNotification(opening
                    ? qsTr("toast_open_failed").arg(nextRestore.error)
                    : saving
                      ? qsTr("toast_export_failed").arg(nextRestore.error)
                      : qsTr("toast_restore_failed").arg(nextRestore.error));
            }
        }
        root.restoreStatus = nextRestore;
    }

    onSelectedRuleIdChanged: root.applyRuleFilter()

    onSelectedSnapshotIdChanged: {
        browserPanel.clearSelection();
        browserPanel.loadSnapshot(selectedSnapshotId);
    }

    globalDrawer: Kirigami.GlobalDrawer {
        title: qsTr("app_title")

        actions: [
            Kirigami.Action {
                text: qsTr("action_back_up_now")
                icon.name: "document-save"
                enabled: !root.busy
                onTriggered: root.requestBackup("")
            },
            Kirigami.Action {
                text: qsTr("action_backup_rules")
                icon.name: "document-properties"
                enabled: !root.busy
                onTriggered: root.openRulesEditor()
            },
            Kirigami.Action {
                text: qsTr("action_restore")
                icon.name: "document-revert"
                enabled: browserPanel.selectionCount > 0 && !root.busy
                onTriggered: restoreDialog.openFor(root.selectedSnapshotId,
                                                    Object.keys(browserPanel.selectedPaths))
            },
            Kirigami.Action {
                text: qsTr("action_save_copies")
                icon.name: "document-save-as"
                enabled: browserPanel.selectionCount > 0 && !root.busy
                onTriggered: browserPanel.saveSelected()
            },
            Kirigami.Action {
                text: qsTr("action_open_backup_folder")
                icon.name: "folder-open"
                onTriggered: App.openPath(Store.repoPath())
            },
            Kirigami.Action {
                text: qsTr("action_lock_backups")
                icon.name: "object-locked"
                shortcut: "Ctrl+L"
                enabled: root.unlockedRules.length > 0
                onTriggered: root.lockBackups()
            },
            Kirigami.Action { separator: true },
            Kirigami.Action {
                text: qsTr("action_settings")
                icon.name: "configure"
                shortcut: "Ctrl+,"
                onTriggered: settingsDialog.open()
            },
            Kirigami.Action {
                text: qsTr("action_about")
                icon.name: "help-about"
                onTriggered: aboutDialog.open()
            },
            Kirigami.Action {
                text: qsTr("action_quit")
                icon.name: "application-exit"
                shortcut: "Ctrl+Q"
                onTriggered: App.quit()
            }
        ]
    }

    pageStack.initialPage: Kirigami.Page {
        id: mainPage

        padding: 0
        title: qsTr("app_title")

        actions: [
            Kirigami.Action {
                text: qsTr("action_back_up_now")
                icon.name: "document-save"
                enabled: !root.busy
                onTriggered: root.requestBackup("")
            },
            Kirigami.Action {
                text: qsTr("action_backup_rules")
                icon.name: "document-properties"
                enabled: !root.busy
                onTriggered: root.openRulesEditor()
            },
            Kirigami.Action {
                text: qsTr("action_settings")
                icon.name: "configure"
                onTriggered: settingsDialog.open()
            }
        ]

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            RowLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0

                RulesPanel {
                    id: rulesPanel
                    Layout.preferredWidth: Math.max(200, Math.min(260, root.width * 0.27))
                    Layout.fillHeight: true
                    items: root.ruleRows
                    selectedId: root.selectedRuleId
                    activeRuleId: root.backupStatus.ruleId
                    backupRunning: root.backupStatus.running
                    busy: root.busy
                    menuEnabled: !root.modalDialogOpen
                    onRuleActivated: (id) => root.activateRule(id)
                    onRuleSelected: (id) => root.selectedRuleId = id
                    onRunRequested: (id) => root.requestBackup(id)
                    onStopRequested: Backup.cancel()
                    onEditRequested: (id) => root.requestEditRule(id)
                    onToggleRequested: (id, enabled) => Config.setRuleEnabled(id, enabled)
                    onDeleteRequested: (id, name) => deleteRuleDialog.openFor(id, name)
                    onAddRequested: root.addRule()
                }

                Rectangle {
                    Layout.fillHeight: true
                    Layout.preferredWidth: 1
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                   Kirigami.Theme.textColor.b, 0.15)
                }

                TimelinePanel {
                    id: timelinePanel
                    Layout.preferredWidth: Math.max(200, Math.min(280, root.width * 0.29))
                    Layout.fillHeight: true
                    items: root.timelineItems
                    heading: root.selectedRuleName
                    emptyText: root.selectedRuleId === ""
                                ? qsTr("timeline_empty") : qsTr("timeline_empty_rule")
                    showRuleName: root.selectedRuleIsOther
                    selectedId: root.selectedSnapshotId
                    loading: root.snapshotsLoading
                    locked: root.selectedRuleLocked
                    hasPassword: root.selectedRuleHasPassword
                    menuEnabled: !root.modalDialogOpen
                    onSnapshotActivated: (id) => root.selectedSnapshotId = id
                    onPinRequested: (id, pinned) => Store.setPinned(id, pinned)
                    onDeleteRequested: (id) => deleteDialog.openFor(id)
                    onUnlockRequested: root.unlockSelectedRule()
                }

                Rectangle {
                    Layout.fillHeight: true
                    Layout.preferredWidth: 1
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                   Kirigami.Theme.textColor.b, 0.15)
                }

                SnapshotBrowser {
                    id: browserPanel
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    busy: root.busy
                    menuEnabled: !root.modalDialogOpen
                    snapshotDate: root.selectedSnapshot() ? root.selectedSnapshot().dateText : ""
                    snapshotTime: root.selectedSnapshot() ? root.selectedSnapshot().timeText : ""
                    snapshotLatest: root.selectedSnapshot() ? root.selectedSnapshot().isLatest : false
                    onRestoreRequested: restoreDialog.openFor(root.selectedSnapshotId,
                                                              Object.keys(browserPanel.selectedPaths))
                    onExportRequested: (destination) => {
                        Restore.saveToFolder(root.selectedSnapshotId,
                                             Object.keys(browserPanel.selectedPaths).join("\n"),
                                             destination);
                        root.restoreStatus = Restore.poll();
                    }
                    onOpenRequested: (relative) => {
                        Restore.openItem(root.selectedSnapshotId, relative);
                        root.restoreStatus = Restore.poll();
                        // A missing xdg-open/repository can fail before the
                        // polling timer ever sees the job running.
                        if (!root.restoreStatus.running
                                && root.restoreStatus.state === "failed")
                            root.showPassiveNotification(
                                qsTr("toast_open_failed").arg(root.restoreStatus.error));
                    }
                    onLoadFailed: (message) => root.showPassiveNotification(
                                       qsTr("browser_load_failed").arg(message))
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: root.busy ? 56 : 30
                color: Kirigami.Theme.backgroundColor
                border.width: 0

                Rectangle {
                    anchors.top: parent.top
                    width: parent.width
                    height: 1
                    color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                   Kirigami.Theme.textColor.b, 0.15)
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: 12
                    spacing: 10
                    visible: root.busy

                    ProgressBar {
                        Layout.fillWidth: true
                        from: 0
                        to: 100
                        value: Math.max(0, root.currentProgress)
                        indeterminate: root.currentProgress <= 0
                    }
                    Label {
                        text: root.statusText()
                        font.pixelSize: 12
                        elide: Text.ElideRight
                        Layout.maximumWidth: 380
                        color: Kirigami.Theme.textColor
                    }
                    Button {
                        text: qsTr("action_cancel")
                        onClicked: {
                            Backup.cancel();
                            Restore.cancel();
                        }
                    }
                }

                Label {
                    anchors.centerIn: parent
                    visible: !root.busy
                    text: root.snapshotsLoading ? qsTr("status_working") : root.statusSummary()
                    font.pixelSize: 12
                    opacity: 0.7
                    elide: Text.ElideRight
                    width: parent.width - 24
                    horizontalAlignment: Text.AlignHCenter
                    color: Kirigami.Theme.textColor
                }
            }
        }
    }

    BackupDialog {
        id: backupDialog
        onPasswordRequested: (ruleId, mode) => {
            const rule = root.ruleById(ruleId);
            if (rule === null)
                return;
            root.pendingAction = "";
            if (Config.passwordMode === "common" || mode === "common") {
                root.openCommonPassword();
                return;
            }
            const changing = mode === "change" && rule.passwordSet;
            const authRequired = changing
                    ? (Store.status().hasRepo && !Store.isRuleUnlocked(ruleId))
                    : (Store.status().hasRepo && !Store.anyUnlocked()
                       && !Store.canAuthorizeWithCredential());
            const name = root.ruleDisplayName(rule);
            if (changing)
                passwordDialog.openChange(ruleId, name, authRequired);
            else
                passwordDialog.openCreate(ruleId, name, authRequired, false);
        }
    }
    UnlockDialog {
        id: passwordDialog
        onFinished: (ruleId, mode) => root.finishPassword(ruleId, mode)
        // A cancelled dialog must not run the backup that was waiting for it.
        onClosed: {
            if (!passwordDialog.succeeded)
                root.pendingAction = "";
        }
    }
    RestoreDialog { id: restoreDialog }
    SettingsDialog {
        id: settingsDialog
        onCommonPasswordRequested: root.openCommonPassword()
        onRulesRequested: {
            settingsDialog.close();
            root.openRulesEditor();
        }
    }

    // Rule edits are saved by Config itself; reinstall the matching systemd
    // timers right away so the schedule follows the rules.
    Connections {
        target: Config
        // qmetaobject signals keep their snake_case name in QML: the handler
        // is onRules_changed, not onRulesChanged.
        function onRules_changed() {
            Store.reloadConfig();
            root.rebuildRuleRows();
            root.applyRuleFilter();
            root.hasRepo = Store.status().hasRepo;
            Scheduler.apply();
        }
        function onRepo_changed() {
            Store.reloadConfig();
            Store.clearCache();
            root.selectedSnapshotId = "";
            root.snapshots = [];
            root.ruleSnapshots = [];
            root.timelineItems = [];
            root.hasRepo = Store.status().hasRepo;
            root.rebuildRuleRows();
            root.applyRuleFilter();
            root.refreshSnapshots();
        }
    }
    AboutDialog { id: aboutDialog }

    // Always polling: jobs can be started from the browser and the restore
    // dialog too, not only from the main window.
    Timer {
        interval: 250
        repeat: true
        running: true
        onTriggered: root.handleFinishedJobs()
    }

    Timer {
        interval: 120
        repeat: true
        running: root.snapshotsLoading
        onTriggered: root.pollSnapshots()
    }

    Dialog {
        id: deleteRuleDialog

        parent: Overlay.overlay
        modal: true
        title: qsTr("delete_rule_title")
        width: Math.min(480, (Overlay.overlay ? Overlay.overlay.width : 800) - 32)
        x: Math.max(16, (Overlay.overlay.width - width) / 2)
        y: Math.max(16, (Overlay.overlay.height - height) / 2)
        z: 2000

        property string ruleId: ""
        property string ruleName: ""

        function openFor(id, name) {
            ruleId = id;
            ruleName = name;
            open();
        }

        contentItem: ColumnLayout {
            spacing: 16

            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: qsTr("delete_rule_message").arg(deleteRuleDialog.ruleName)
                color: Kirigami.Theme.textColor
                font.pixelSize: 14
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Item { Layout.fillWidth: true }
                Button {
                    text: qsTr("action_cancel")
                    onClicked: deleteRuleDialog.close()
                }
                Button {
                    text: qsTr("action_delete")
                    icon.name: "edit-delete"
                    onClicked: {
                        const id = deleteRuleDialog.ruleId;
                        deleteRuleDialog.close();
                        Config.removeRule(id);
                    }
                }
            }
        }
    }

    Dialog {
        id: deleteDialog

        parent: Overlay.overlay
        modal: true
        title: qsTr("delete_snapshot_title")
        width: 440
        x: Math.max(16, (Overlay.overlay.width - width) / 2)
        y: Math.max(16, (Overlay.overlay.height - height) / 2)
        z: 2000

        property string snapshotId: ""

        function openFor(id) {
            snapshotId = id;
            open();
        }

        contentItem: ColumnLayout {
            spacing: 16

            Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: qsTr("delete_snapshot_message").arg(deleteDialog.snapshotId.substring(0, 8))
                color: Kirigami.Theme.textColor
                font.pixelSize: 14
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Item { Layout.fillWidth: true }
                Button {
                    text: qsTr("action_cancel")
                    onClicked: deleteDialog.close()
                }
                Button {
                    text: qsTr("action_delete")
                    icon.name: "edit-delete"
                    onClicked: {
                        deleteDialog.close();
                        root.removeSnapshot(deleteDialog.snapshotId);
                    }
                }
            }
        }
    }

    // Development-only: `kustos --capture [--capture-dialog settings|about|restore]`
    // renders the window to /tmp/kustos-capture.png and quits.
    property string captureFile: typeof CapturePath !== "undefined" ? CapturePath : ""
    property string captureDialog: typeof CaptureDialog !== "undefined" ? CaptureDialog : ""
    // Development-only: start a backup in a headless capture, to exercise the
    // real GUI path (including credential resolution) without a click.
    property string captureAction: typeof CaptureAction !== "undefined" ? CaptureAction : ""

    function captureRequested() {
        return root.captureFile !== "";
    }

    function captureDialogName() {
        return root.captureDialog;
    }

    function openDialogForCapture(name) {
        if (name === "settings")
            settingsDialog.open();
        else if (name === "security") {
            settingsDialog.currentCategory = "security";
            settingsDialog.open();
        } else if (name === "maintenance") {
            settingsDialog.currentCategory = "maintenance";
            settingsDialog.open();
        } else if (name === "retention") {
            settingsDialog.currentCategory = "retention";
            settingsDialog.open();
        } else if (name === "about")
            aboutDialog.open();
        else if (name === "rules" || name === "backup")
            root.openRulesEditor();
        else if (name === "key" || name === "password") {
            const rule = root.firstEnabledRule();
            if (rule !== null) {
                const ruleNameText = root.ruleDisplayName(rule);
                if (Config.passwordMode === "common") {
                    if (Config.commonPasswordSet)
                        passwordDialog.openUnlock(rule.id, ruleNameText, true);
                    else
                        passwordDialog.openCommon(rule.id, ruleNameText, false);
                } else if (rule.passwordSet && !Store.isRuleUnlocked(rule.id)) {
                    passwordDialog.openUnlock(rule.id, ruleNameText, false);
                } else {
                    passwordDialog.openCreate(rule.id, ruleNameText,
                                              Store.status().hasRepo && !Store.anyUnlocked()
                                              && !Store.canAuthorizeWithCredential(), false);
                }
            }
        } else if (name === "restore" && root.selectedSnapshotId !== "")
            restoreDialog.openFor(root.selectedSnapshotId, []);
    }

    Component.onCompleted: {
        // The rule list comes from the configuration, not from the repository,
        // so it is available even while every backup is locked.
        root.rebuildRuleRows();
        root.applyRuleFilter();
        root.lockGeneration = Store.lockGeneration();
        root.hasRepo = Store.status().hasRepo;
        if (Store.anyUnlocked())
            root.refreshSnapshots();
        else {
            root.snapshots = [];
            root.snapshotsLoading = false;
        }
        root.backupStatus = Backup.poll();
        root.restoreStatus = Restore.poll();
        if (root.captureRequested()) {
            root.openDialogForCapture(root.captureDialogName());
            if (root.captureAction === "backup")
                root.requestBackup("");
            else if (root.captureAction.indexOf("backup-rule:") === 0)
                root.requestBackup(root.captureAction.substring("backup-rule:".length));
            else if (root.captureAction === "check")
                Maintenance.start(false);

        }
    }
}
