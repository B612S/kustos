// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Dialog {
    id: dialog

    parent: Overlay.overlay
    modal: true
    title: qsTr("about_title")
    padding: 18

    width: Math.min(560, (Overlay.overlay ? Overlay.overlay.width : 800) - 40)
    x: Math.max(12, (Overlay.overlay.width - width) / 2)
    y: Math.max(12, (Overlay.overlay.height - height) / 2)

    property bool keyVisible: false

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
        spacing: 12

        RowLayout {
            spacing: 14

            Image {
                source: "qrc:/assets/kustos.png"
                sourceSize.width: 64
                sourceSize.height: 64
                Layout.preferredWidth: 64
                Layout.preferredHeight: 64
            }
            ColumnLayout {
                spacing: 2
                Label {
                    text: "Kustos"
                    font.pixelSize: 22
                    font.weight: Font.Bold
                    color: Kirigami.Theme.textColor
                }
                Label {
                    text: qsTr("about_version").arg(App.version())
                    font.pixelSize: 12
                    opacity: 0.7
                    color: Kirigami.Theme.textColor
                }
            }
        }

        Label {
            Layout.fillWidth: true
            text: qsTr("about_description")
            wrapMode: Text.WordWrap
            font.pixelSize: 13
            color: Kirigami.Theme.textColor
        }

        GridLayout {
            columns: 2
            columnSpacing: 12
            rowSpacing: 8
            Layout.fillWidth: true

            Label {
                text: qsTr("about_repo")
                font.pixelSize: 12
                opacity: 0.7
                color: Kirigami.Theme.textColor
            }
            RowLayout {
                Layout.fillWidth: true
                Label {
                    Layout.fillWidth: true
                    text: Store.repoPath()
                    font.pixelSize: 12
                    elide: Text.ElideMiddle
                    color: Kirigami.Theme.textColor
                }
                ToolButton {
                    icon.name: "folder-open"
                    onClicked: App.openPath(Store.repoPath())
                }
            }

            Label {
                text: qsTr("about_config")
                font.pixelSize: 12
                opacity: 0.7
                color: Kirigami.Theme.textColor
            }
            RowLayout {
                Layout.fillWidth: true
                Label {
                    Layout.fillWidth: true
                    text: App.configPath()
                    font.pixelSize: 12
                    elide: Text.ElideMiddle
                    color: Kirigami.Theme.textColor
                }
                ToolButton {
                    icon.name: "text-plain"
                    onClicked: App.openPath(App.configPath())
                }
            }

            Label {
                text: qsTr("about_log")
                font.pixelSize: 12
                opacity: 0.7
                color: Kirigami.Theme.textColor
            }
            RowLayout {
                Layout.fillWidth: true
                Label {
                    Layout.fillWidth: true
                    text: App.logPath()
                    font.pixelSize: 12
                    elide: Text.ElideMiddle
                    color: Kirigami.Theme.textColor
                }
                ToolButton {
                    icon.name: "text-plain"
                    onClicked: App.openPath(App.logPath())
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                           Kirigami.Theme.textColor.b, 0.15)
        }

        Label {
            Layout.fillWidth: true
            text: qsTr("about_key")
            font.pixelSize: 13
            font.weight: Font.DemiBold
            color: Kirigami.Theme.textColor
        }
        Label {
            Layout.fillWidth: true
            text: dialog.keyVisible ? Store.password() : "••••••••••••••••"
            font.pixelSize: 13
            font.family: "monospace"
            wrapMode: Text.WrapAnywhere
            color: Kirigami.Theme.textColor
        }
        RowLayout {
            spacing: 8
            Button {
                text: dialog.keyVisible ? qsTr("action_hide") : qsTr("action_show")
                onClicked: dialog.keyVisible = !dialog.keyVisible
            }
            Button {
                text: qsTr("action_copy_key")
                icon.name: "edit-copy"
                onClicked: App.copyText(Store.password())
            }
        }
        Label {
            Layout.fillWidth: true
            text: qsTr("about_key_hint")
            wrapMode: Text.WordWrap
            font.pixelSize: 11
            opacity: 0.7
            color: Kirigami.Theme.textColor
        }

        Label {
            Layout.fillWidth: true
            text: qsTr("about_credits")
            wrapMode: Text.WordWrap
            font.pixelSize: 11
            opacity: 0.6
            color: Kirigami.Theme.textColor
        }
        Label {
            Layout.fillWidth: true
            text: qsTr("about_license")
            font.pixelSize: 11
            opacity: 0.6
            color: Kirigami.Theme.textColor
        }

        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            Button {
                text: qsTr("action_close")
                onClicked: dialog.close()
            }
        }
    }
}
