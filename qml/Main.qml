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
    property var timelineItems: []
    property string selectedSnapshotId: ""

    property var backupStatus: ({
        running: false, percent: 0, state: "idle", error: "",
        speed: "", eta: "", files: 0, added: 0, snapshotId: "", current: ""
    })
    property var restoreStatus: ({
        running: false, percent: 0, state: "idle", error: "", current: ""
    })
    property var schedulerStatus: ({ enabled: false, state: "absent", error: "", minutes: 60 })

    readonly property bool busy: backupStatus.running || restoreStatus.running
    readonly property int currentProgress: restoreStatus.running ? restoreStatus.percent : backupStatus.percent

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
        for (let i = 0; i < snapshots.length; ++i) {
            const snapshot = snapshots[i];
            if (snapshot.dayKey !== lastDay) {
                lastDay = snapshot.dayKey;
                items.push({ header: true, label: root.dayLabel(snapshot.dayKey) });
            }
            items.push({ header: false, snapshot: snapshot });
        }
        timelineItems = items;
    }

    function ensureSelection() {
        if (snapshots.length === 0) {
            selectedSnapshotId = "";
            return;
        }
        let found = false;
        for (let i = 0; i < snapshots.length; ++i) {
            if (snapshots[i].id === selectedSnapshotId)
                found = true;
        }
        if (!found)
            selectedSnapshotId = snapshots[0].id;
    }

    function selectedSnapshot() {
        for (let i = 0; i < snapshots.length; ++i) {
            if (snapshots[i].id === selectedSnapshotId)
                return snapshots[i];
        }
        return null;
    }

    function refreshSnapshots() {
        snapshots = Store.snapshots();
        rebuildTimeline();
        ensureSelection();
    }

    function startBackup() {
        Backup.start();
        backupStatus = Backup.poll();
    }

    function removeSnapshot(id) {
        Store.remove(id);
        refreshSnapshots();
    }

    function statusText() {
        if (restoreStatus.running) {
            if (restoreStatus.state === "cancelling")
                return qsTr("status_cancelling");
            return qsTr("status_restoring").arg(restoreStatus.current);
        }
        if (backupStatus.running) {
            if (backupStatus.state === "preparing")
                return qsTr("status_preparing");
            if (backupStatus.state === "cancelling")
                return qsTr("status_cancelling");
            if (backupStatus.speed === "")
                return qsTr("status_working");
            return qsTr("status_copying").arg(backupStatus.speed).arg(backupStatus.eta);
        }
        return "";
    }

    function statusSummary() {
        return qsTr("status_summary")
                .arg(snapshots.length)
                .arg(Store.repoPath())
                .arg(App.formatBytes(Store.freeBytes()));
    }

    function handleFinishedJobs() {
        const nextBackup = Backup.poll();
        const previousRunning = root.backupStatus.running;
        if (previousRunning && !nextBackup.running) {
            if (nextBackup.state === "done") {
                root.selectedSnapshotId = nextBackup.snapshotId;
                root.refreshSnapshots();
                root.showPassiveNotification(qsTr("toast_backup_done"));
            } else if (nextBackup.state === "failed") {
                root.showPassiveNotification(qsTr("toast_backup_failed").arg(nextBackup.error));
            }
        }
        root.backupStatus = nextBackup;

        const nextRestore = Restore.poll();
        const restoring = root.restoreStatus.running;
        if (restoring && !nextRestore.running) {
            if (nextRestore.state === "done")
                root.showPassiveNotification(qsTr("toast_restore_done").arg(nextRestore.added));
            else if (nextRestore.state === "failed")
                root.showPassiveNotification(qsTr("toast_restore_failed").arg(nextRestore.error));
        }
        root.restoreStatus = nextRestore;
    }

    onSelectedSnapshotIdChanged: {
        browserPanel.clearSelection();
        browserPanel.loadSnapshot(selectedSnapshotId);
    }

    globalDrawer: Kirigami.GlobalDrawer {
        title: qsTr("app_title")
        titleIcon: "qrc:/assets/kustos.png"

        actions: [
            Kirigami.Action {
                text: qsTr("action_back_up_now")
                icon.name: "document-save"
                enabled: !root.busy
                onTriggered: backupDialog.open()
            },
            Kirigami.Action {
                text: qsTr("action_restore")
                icon.name: "document-revert"
                enabled: browserPanel.selectionCount > 0 && !root.busy
                onTriggered: restoreDialog.openFor(root.selectedSnapshotId,
                                                    Object.keys(browserPanel.selectedPaths))
            },
            Kirigami.Action {
                text: qsTr("action_open_backup_folder")
                icon.name: "folder-open"
                onTriggered: App.openPath(Store.repoPath())
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
                onTriggered: backupDialog.open()
            },
            Kirigami.Action {
                text: qsTr("action_restore")
                icon.name: "document-revert"
                enabled: browserPanel.selectionCount > 0 && !root.busy
                onTriggered: restoreDialog.openFor(root.selectedSnapshotId,
                                                    Object.keys(browserPanel.selectedPaths))
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

                TimelinePanel {
                    id: timelinePanel
                    Layout.preferredWidth: 310
                    Layout.fillHeight: true
                    items: root.timelineItems
                    selectedId: root.selectedSnapshotId
                    onSnapshotActivated: (id) => root.selectedSnapshotId = id
                    onDeleteRequested: (id) => deleteDialog.openFor(id)
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
                    snapshotDate: root.selectedSnapshot() ? root.selectedSnapshot().dateText : ""
                    snapshotTime: root.selectedSnapshot() ? root.selectedSnapshot().timeText : ""
                    snapshotLatest: root.selectedSnapshot() ? root.selectedSnapshot().isLatest : false
                    onRestoreRequested: restoreDialog.openFor(root.selectedSnapshotId,
                                                              Object.keys(browserPanel.selectedPaths))
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
                        onClicked: Restore.cancel()
                    }
                }

                Label {
                    anchors.centerIn: parent
                    visible: !root.busy
                    text: root.statusSummary()
                    font.pixelSize: 12
                    opacity: 0.7
                    elide: Text.ElideRight
                    width: parent.width - 24
                    horizontalAlignment: Text.AlignHCenter
                    color: Kirigami.Theme.textColor
                }
            }
        }

        Kirigami.PlaceholderMessage {
            anchors.centerIn: parent
            width: parent.width - 120
            visible: root.snapshots.length === 0
            icon.name: "document-save"
            text: qsTr("empty_title")
            explanation: qsTr("empty_hint")
            helpfulAction: Kirigami.Action {
                text: qsTr("action_back_up_now")
                icon.name: "document-save"
                enabled: !root.busy
                onTriggered: backupDialog.open()
            }
        }
    }

    BackupDialog {
        id: backupDialog
        onStartRequested: root.startBackup()
    }
    RestoreDialog { id: restoreDialog }
    SettingsDialog {
        id: settingsDialog
        onScopeRequested: {
            settingsDialog.close();
            backupDialog.open();
        }
    }
    AboutDialog { id: aboutDialog }

    Timer {
        interval: 250
        repeat: true
        running: root.busy
        onTriggered: root.handleFinishedJobs()
    }

    Timer {
        interval: 1500
        repeat: true
        running: true
        onTriggered: root.schedulerStatus = Scheduler.poll()
    }

    Dialog {
        id: deleteDialog

        parent: Overlay.overlay
        modal: true
        title: qsTr("delete_snapshot_title")
        width: 440
        x: Math.max(16, (Overlay.overlay.width - width) / 2)
        y: Math.max(16, (Overlay.overlay.height - height) / 2)

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

    function captureRequested() {
        return root.captureFile !== "";
    }

    function captureDialogName() {
        return root.captureDialog;
    }

    function openDialogForCapture(name) {
        if (name === "settings")
            settingsDialog.open();
        else if (name === "about")
            aboutDialog.open();
        else if (name === "backup")
            backupDialog.open();
        else if (name === "restore" && root.selectedSnapshotId !== "")
            restoreDialog.openFor(root.selectedSnapshotId, []);
    }

    Component.onCompleted: {
        root.refreshSnapshots();
        root.schedulerStatus = Scheduler.poll();
        root.backupStatus = Backup.poll();
        root.restoreStatus = Restore.poll();
        if (root.captureRequested())
            root.openDialogForCapture(root.captureDialogName());
    }
}
