// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.kde.kirigami as Kirigami

// Restore wizard: destination and conflict policy, then the per-item progress
// reported by the Rust restore job.
Dialog {
    id: dialog

    parent: Overlay.overlay
    modal: true
    focus: true
    title: qsTr("restore_title")
    padding: 18
    closePolicy: Popup.CloseOnEscape
    z: 2000

    width: Math.min(640, (Overlay.overlay ? Overlay.overlay.width : 800) - 40)
    x: Math.max(12, (Overlay.overlay.width - width) / 2)
    y: Math.max(12, Math.min((Overlay.overlay.height - height) / 2,
                             Overlay.overlay.height - height - 12))

    property string snapshotId: ""
    property var itemPaths: []
    property string destination: ""
    property string policy: "overwrite"
    property var status: ({ running: false, percent: 0, state: "", error: "", current: "", added: 0 })
    readonly property bool running: status.running

    function urlToPath(url) {
        return decodeURIComponent(url.toString().replace(/^file:\/\//, ""));
    }

    function openFor(id, paths) {
        if (!paths || paths.length === 0 || id === "")
            return;
        snapshotId = id;
        itemPaths = paths;
        destination = "";
        policy = "overwrite";
        status = ({ running: false, percent: 0, state: "", error: "", current: "", added: 0 });
        open();
    }

    background: Rectangle {
        color: Qt.rgba(Kirigami.Theme.backgroundColor.r,
                       Kirigami.Theme.backgroundColor.g,
                       Kirigami.Theme.backgroundColor.b, 1.0)
        radius: 8
        border.width: 1
        border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                              Kirigami.Theme.textColor.b, 0.15)
    }

    contentItem: ColumnLayout {
        spacing: 14

        Label {
            Layout.fillWidth: true
            text: qsTr("restore_count").arg(dialog.itemPaths.length)
            font.pixelSize: 15
            font.weight: Font.DemiBold
            color: Kirigami.Theme.textColor
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Repeater {
                model: Math.min(dialog.itemPaths.length, 6)
                delegate: Label {
                    Layout.fillWidth: true
                    text: "• " + String(dialog.itemPaths[index]).replace(Browser.homeRoot(), "~")
                    font.pixelSize: 13
                    elide: Text.ElideMiddle
                    opacity: 0.85
                    color: Kirigami.Theme.textColor
                }
            }
            Label {
                visible: dialog.itemPaths.length > 6
                text: qsTr("restore_more").arg(dialog.itemPaths.length - 6)
                font.pixelSize: 12
                opacity: 0.6
                color: Kirigami.Theme.textColor
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                           Kirigami.Theme.textColor.b, 0.15)
        }

        GridLayout {
            columns: 2
            columnSpacing: 16
            rowSpacing: 12
            Layout.fillWidth: true

            Label {
                text: qsTr("restore_destination")
                Layout.fillWidth: true
                font.pixelSize: 14
                color: Kirigami.Theme.textColor
            }
            RowLayout {
                Layout.preferredWidth: 260
                spacing: 4

                ComboBox {
                    Layout.fillWidth: true
                    font.pixelSize: 14
                    model: [
                        { value: "original", text: qsTr("restore_original") },
                        { value: "folder", text: qsTr("restore_folder") }
                    ]
                    textRole: "text"
                    valueRole: "value"
                    currentIndex: dialog.destination === "" ? 0 : 1
                    onActivated: (index) => {
                        if (model[index].value === "folder")
                            folderDialog.open();
                        else
                            dialog.destination = "";
                    }
                }
                ToolButton {
                    icon.name: "folder-open"
                    onClicked: folderDialog.open()
                    ToolTip.text: qsTr("restore_choose_folder")
                    ToolTip.visible: hovered
                }
            }

            Label {
                visible: dialog.destination !== ""
                Layout.fillWidth: true
                text: qsTr("restore_into")
                font.pixelSize: 14
                color: Kirigami.Theme.textColor
            }
            Label {
                visible: dialog.destination !== ""
                Layout.preferredWidth: 260
                text: dialog.destination
                font.pixelSize: 12
                elide: Text.ElideMiddle
                opacity: 0.75
                color: Kirigami.Theme.textColor
            }

            Label {
                text: qsTr("restore_conflict")
                Layout.fillWidth: true
                font.pixelSize: 14
                color: Kirigami.Theme.textColor
            }
            ComboBox {
                Layout.preferredWidth: 260
                font.pixelSize: 14
                model: [
                    { value: "overwrite", text: qsTr("restore_policy_overwrite") },
                    { value: "skip", text: qsTr("restore_policy_skip") },
                    { value: "keep_both", text: qsTr("restore_policy_keep_both") }
                ]
                textRole: "text"
                valueRole: "value"
                currentIndex: 0
                onActivated: (index) => { dialog.policy = model[index].value; }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 6
            visible: dialog.running

            ProgressBar {
                Layout.fillWidth: true
                from: 0
                to: 100
                value: dialog.status.percent
            }
            Label {
                Layout.fillWidth: true
                text: String(dialog.status.current).replace(Browser.homeRoot(), "~")
                font.pixelSize: 12
                elide: Text.ElideMiddle
                opacity: 0.7
                color: Kirigami.Theme.textColor
            }
        }

        Label {
            Layout.fillWidth: true
            visible: dialog.status.error !== ""
            wrapMode: Text.WordWrap
            text: dialog.status.error
            color: Kirigami.Theme.negativeTextColor
            font.pixelSize: 12
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Item { Layout.fillWidth: true }

            Button {
                text: qsTr("action_cancel")
                enabled: !dialog.running
                onClicked: dialog.close()
            }
            Button {
                text: qsTr("action_restore")
                icon.name: "document-revert"
                enabled: !dialog.running && dialog.itemPaths.length > 0
                onClicked: {
                    Restore.start(dialog.snapshotId, dialog.itemPaths.join("\n"),
                                  dialog.destination, dialog.policy);
                    dialog.status = Restore.poll();
                }
            }
        }
    }

    Timer {
        interval: 200
        repeat: true
        running: dialog.status.running
        onTriggered: {
            dialog.status = Restore.poll();
            if (!dialog.status.running && dialog.status.state === "done")
                dialog.close();
        }
    }

    FolderDialog {
        id: folderDialog
        title: qsTr("restore_choose_folder")
        onAccepted: dialog.destination = dialog.urlToPath(selectedFolder)
    }
}
