// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.kde.kirigami as Kirigami

// "What to back up": the editable scope of every backup — included folders
// and paths excluded from them. Changes are saved immediately (the same list
// as in Settings), so the next run uses them.
Dialog {
    id: dialog

    parent: Overlay.overlay
    modal: true
    focus: true
    title: qsTr("backup_title")
    padding: 18
    closePolicy: Popup.CloseOnEscape

    width: Math.min(660, (Overlay.overlay ? Overlay.overlay.width : 800) - 40)
    height: Math.min(560, (Overlay.overlay ? Overlay.overlay.height : 600) - 40)
    x: Math.max(12, (Overlay.overlay.width - width) / 2)
    y: Math.max(12, (Overlay.overlay.height - height) / 2)

    property var sources: []
    property var excludes: []

    signal startRequested()

    function refresh() {
        sources = Config.sources();
        excludes = Config.excludes();
    }

    function pretty(path) {
        return String(path).replace(App.homePath(), "~");
    }

    function urlToPath(url) {
        return decodeURIComponent(url.toString().replace(/^file:\/\//, ""));
    }

    onOpened: refresh()

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
            text: qsTr("backup_include_title")
            font.pixelSize: 15
            font.weight: Font.DemiBold
            color: Kirigami.Theme.textColor
        }
        Label {
            Layout.fillWidth: true
            text: qsTr("backup_include_hint")
            font.pixelSize: 12
            wrapMode: Text.WordWrap
            opacity: 0.7
            color: Kirigami.Theme.textColor
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(170, 34 + dialog.sources.length * 30)
            clip: true

            ColumnLayout {
                width: parent.width
                spacing: 2

                Repeater {
                    model: dialog.sources

                    delegate: RowLayout {
                        Layout.fillWidth: true
                        spacing: 6

                        Label {
                            Layout.fillWidth: true
                            text: "• " + dialog.pretty(modelData)
                            font.pixelSize: 13
                            elide: Text.ElideMiddle
                            color: Kirigami.Theme.textColor
                        }
                        ToolButton {
                            icon.name: "list-remove"
                            onClicked: {
                                Config.removeSource(index);
                                dialog.refresh();
                            }
                        }
                    }
                }
            }
        }

        RowLayout {
            spacing: 8
            Button {
                text: qsTr("backup_add_folder")
                icon.name: "folder-new"
                onClicked: includeFolderDialog.open()
            }
            Button {
                text: qsTr("backup_add_file")
                icon.name: "document-new"
                onClicked: includeFileDialog.open()
            }
            Item { Layout.fillWidth: true }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                           Kirigami.Theme.textColor.b, 0.15)
        }

        Label {
            Layout.fillWidth: true
            text: qsTr("backup_exclude_title")
            font.pixelSize: 15
            font.weight: Font.DemiBold
            color: Kirigami.Theme.textColor
        }
        Label {
            Layout.fillWidth: true
            text: qsTr("backup_exclude_hint")
            font.pixelSize: 12
            wrapMode: Text.WordWrap
            opacity: 0.7
            color: Kirigami.Theme.textColor
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(130, 34 + dialog.excludes.length * 30)
            clip: true

            ColumnLayout {
                width: parent.width
                spacing: 2

                Repeater {
                    model: dialog.excludes

                    delegate: RowLayout {
                        Layout.fillWidth: true
                        spacing: 6

                        Label {
                            Layout.fillWidth: true
                            text: "− " + dialog.pretty(modelData)
                            font.pixelSize: 13
                            elide: Text.ElideMiddle
                            color: Kirigami.Theme.textColor
                        }
                        ToolButton {
                            icon.name: "list-remove"
                            onClicked: {
                                Config.removeExclude(index);
                                dialog.refresh();
                            }
                        }
                    }
                }
            }
        }

        Button {
            text: qsTr("backup_add_exclude")
            icon.name: "folder-new"
            onClicked: excludeFolderDialog.open()
        }

        Label {
            Layout.fillWidth: true
            visible: dialog.sources.length === 0
            text: qsTr("backup_no_sources")
            wrapMode: Text.WordWrap
            font.pixelSize: 12
            color: Kirigami.Theme.negativeTextColor
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Item { Layout.fillWidth: true }
            Button {
                text: qsTr("action_cancel")
                onClicked: dialog.close()
            }
            Button {
                text: qsTr("action_back_up_now")
                icon.name: "document-save"
                enabled: dialog.sources.length > 0
                onClicked: {
                    dialog.close();
                    dialog.startRequested();
                }
            }
        }
    }

    FolderDialog {
        id: includeFolderDialog
        title: qsTr("backup_add_folder")
        onAccepted: {
            Config.addSource(dialog.urlToPath(selectedFolder));
            dialog.refresh();
        }
    }
    FileDialog {
        id: includeFileDialog
        title: qsTr("backup_add_file")
        fileMode: FileDialog.OpenFile
        onAccepted: {
            Config.addSource(dialog.urlToPath(selectedFile));
            dialog.refresh();
        }
    }
    FolderDialog {
        id: excludeFolderDialog
        title: qsTr("backup_add_exclude")
        onAccepted: {
            Config.addExclude(dialog.urlToPath(selectedFolder));
            dialog.refresh();
        }
    }
}
