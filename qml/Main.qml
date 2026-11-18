// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Window parameters follow Luma: a 960x640 Kirigami window with a 640x480
// minimum, a GlobalDrawer for the application menu, and settings as an
// overlay dialog instead of a separate page.
Kirigami.ApplicationWindow {
    id: root

    visible: true
    width: 960
    height: 640
    minimumWidth: 640
    minimumHeight: 480
    title: qsTr("app_title")

    property string selectedSnapshotId: ""

    readonly property bool busy: Backup.running || Restore.running
    readonly property int currentProgress: Restore.running ? Restore.progress : Backup.progress

    function ensureSelection() {
        if (Timeline.count === 0) {
            selectedSnapshotId = "";
            return;
        }
        if (selectedSnapshotId === "" || Timeline.indexOfId(selectedSnapshotId) < 0)
            selectedSnapshotId = Timeline.get(0).id;
    }

    function statusText() {
        if (Restore.running) {
            if (Restore.state === "cancelling")
                return qsTr("status_cancelling");
            return qsTr("status_restoring").arg(Restore.currentItem);
        }
        if (Backup.running) {
            if (Backup.state === "preparing")
                return qsTr("status_preparing");
            if (Backup.state === "cancelling")
                return qsTr("status_cancelling");
            if (Backup.speedText === "")
                return qsTr("status_working");
            return qsTr("status_copying").arg(Backup.speedText).arg(Backup.etaText);
        }
        return "";
    }

    function statusSummary() {
        const free = Store.availableText !== "" ? Store.availableText : "—";
        return qsTr("status_summary").arg(Timeline.count).arg(Store.targetDir).arg(free);
    }

    // Development-only hook for `aion --capture ... --capture-dialog <name>`.
    function openDialogForCapture(name) {
        if (name === "settings" || name.startsWith("settings:")) {
            const parts = name.split(":");
            settingsDialog.currentCategory = parts.length > 1 ? parts[1] : "general";
            settingsDialog.open();
            return;
        }
        if (name === "about") {
            aboutDialog.open();
            return;
        }
        if (name === "restore") {
            const paths = Object.keys(browserPanel.selectedPaths);
            if (paths.length === 0 && Browser.count > 0)
                paths.push(Browser.get(0).relativePath);
            restoreDialog.openFor(root.selectedSnapshotId, paths);
        }
    }

    onSelectedSnapshotIdChanged: {
        browserPanel.clearSelection();
        if (selectedSnapshotId !== "")
            Browser.setSnapshotId(selectedSnapshotId);
    }

    globalDrawer: Kirigami.GlobalDrawer {
        title: qsTr("app_title")
        titleIcon: "qrc:/assets/aion.png"

        actions: [
            Kirigami.Action {
                text: qsTr("action_back_up_now")
                icon.name: "document-save"
                enabled: !root.busy
                onTriggered: Backup.start()
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
                onTriggered: App.openPath(Store.targetDir)
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
                onTriggered: Backup.start()
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
                    onRestoreRequested: restoreDialog.openFor(root.selectedSnapshotId,
                                                              Object.keys(browserPanel.selectedPaths))
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: root.busy ? 56 : 30
                color: Kirigami.Theme.alternativeBackgroundColor

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
                        indeterminate: root.currentProgress < 0
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
                        onClicked: Restore.running ? Restore.cancel() : Backup.cancel()
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
            visible: Timeline.count === 0
            icon.name: "document-save"
            text: qsTr("empty_title")
            explanation: qsTr("empty_hint")
            helpfulAction: Kirigami.Action {
                text: qsTr("action_back_up_now")
                icon.name: "document-save"
                enabled: !root.busy
                onTriggered: Backup.start()
            }
        }
    }

    RestoreDialog { id: restoreDialog }
    SettingsDialog { id: settingsDialog }
    AboutDialog { id: aboutDialog }

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
                text: qsTr("delete_snapshot_message").arg(deleteDialog.snapshotId)
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
                        Store.removeSnapshot(deleteDialog.snapshotId);
                    }
                }
            }
        }
    }

    Connections {
        target: Timeline
        function onCountChanged() { root.ensureSelection(); }
    }

    Connections {
        target: Backup
        function onFinished(ok, snapshotId) {
            if (ok) {
                root.selectedSnapshotId = snapshotId;
                root.showPassiveNotification(qsTr("toast_backup_done"));
            } else if (Backup.state !== "cancelled") {
                root.showPassiveNotification(qsTr("toast_backup_failed").arg(Backup.errorMessage));
            }
            if (ok && Backup.warnings.length > 0)
                root.showPassiveNotification(Backup.warnings.join("\n"));
        }
    }

    Connections {
        target: Restore
        function onFinished(ok, restoredCount) {
            if (ok)
                root.showPassiveNotification(qsTr("toast_restore_done").arg(restoredCount));
            else if (Restore.state !== "cancelled")
                root.showPassiveNotification(qsTr("toast_restore_failed").arg(Restore.errorMessage));
        }
    }

    Component.onCompleted: root.ensureSelection()
}
