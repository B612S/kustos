// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Two modes:
//  - "unlock": ask for the repository key (it is never stored)
//  - "new":    show the key exactly once, right after the repository is created
Dialog {
    id: dialog

    parent: Overlay.overlay
    modal: true
    focus: true
    padding: 18
    closePolicy: Popup.CloseOnEscape

    width: Math.min(540, (Overlay.overlay ? Overlay.overlay.width : 800) - 40)
    x: Math.max(12, (Overlay.overlay.width - width) / 2)
    y: Math.max(12, (Overlay.overlay.height - height) / 2)

    property string mode: "unlock"
    property string key: ""
    property string error: ""

    signal unlocked()
    signal acknowledged()

    title: mode === "new" ? qsTr("key_title") : qsTr("unlock_title")

    function openUnlock() {
        mode = "unlock";
        key = "";
        error = "";
        open();
    }

    function openNew(generated) {
        mode = "new";
        key = generated;
        error = "";
        open();
    }

    background: Rectangle {
        color: Qt.rgba(Kirigami.Theme.backgroundColor.r,
                       Kirigami.Theme.backgroundColor.g,
                       Kirigami.Theme.backgroundColor.b, 0.98)
        radius: 8
        border.width: 1
        border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                              Kirigami.Theme.textColor.b, 0.15)
    }

    contentItem: ColumnLayout {
        spacing: 14

        Label {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: dialog.mode === "new" ? qsTr("key_hint") : qsTr("unlock_hint")
            font.pixelSize: 13
            color: Kirigami.Theme.textColor
        }

        TextField {
            id: keyField
            Layout.fillWidth: true
            visible: dialog.mode === "unlock"
            placeholderText: qsTr("unlock_placeholder")
            echoMode: TextInput.Password
            font.pixelSize: 14
            onAccepted: unlockButton.clicked()
        }

        Label {
            Layout.fillWidth: true
            visible: dialog.mode === "new"
            text: dialog.key
            font.pixelSize: 15
            font.family: "monospace"
            wrapMode: Text.WrapAnywhere
            color: Kirigami.Theme.textColor
        }

        Label {
            Layout.fillWidth: true
            visible: dialog.error !== ""
            text: dialog.error
            wrapMode: Text.WordWrap
            font.pixelSize: 12
            color: Kirigami.Theme.negativeTextColor
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            visible: dialog.mode === "new"

            Button {
                text: qsTr("action_copy")
                icon.name: "edit-copy"
                onClicked: App.copyText(dialog.key)
            }
            CheckBox {
                id: savedBox
                text: qsTr("key_saved")
                font.pixelSize: 13
            }
            Item { Layout.fillWidth: true }
            Button {
                text: qsTr("action_continue")
                enabled: savedBox.checked
                onClicked: {
                    dialog.close();
                    dialog.acknowledged();
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            visible: dialog.mode === "unlock"

            Item { Layout.fillWidth: true }
            Button {
                text: qsTr("action_cancel")
                onClicked: dialog.close()
            }
            Button {
                id: unlockButton
                text: qsTr("action_unlock")
                icon.name: "object-unlocked"
                onClicked: {
                    if (Store.unlock(keyField.text)) {
                        dialog.close();
                        dialog.unlocked();
                    } else {
                        dialog.error = qsTr("unlock_wrong");
                    }
                }
            }
        }
    }
}
