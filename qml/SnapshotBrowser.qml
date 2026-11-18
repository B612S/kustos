// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// File browser for one snapshot: breadcrumbs, a list with selection and the
// restore entry point. It never shows the live home directory, only the
// contents of the selected snapshot.
Rectangle {
    id: panel

    color: Kirigami.Theme.backgroundColor

    // Relative paths selected for restore, used as a set.
    property var selectedPaths: ({})
    readonly property int selectionCount: Object.keys(selectedPaths).length

    signal restoreRequested()

    function isSelected(relative) {
        return selectedPaths[relative] === true;
    }

    function toggle(relative) {
        const copy = Object.assign({}, selectedPaths);
        if (copy[relative])
            delete copy[relative];
        else
            copy[relative] = true;
        selectedPaths = copy;
    }

    function selectAll() {
        const copy = Object.assign({}, selectedPaths);
        for (let i = 0; i < Browser.count; ++i) {
            const entry = Browser.get(i);
            copy[entry.relativePath] = true;
        }
        selectedPaths = copy;
    }

    function clearSelection() {
        selectedPaths = ({});
    }

    function breadcrumbs() {
        const display = Browser.displayPath;
        if (display === "/")
            return [{ label: "/", path: "" }];
        const home = Store.homeRoot;
        const parts = display.split("/");
        const result = [];
        let path = "";
        for (let i = 0; i < parts.length; ++i) {
            if (parts[i] === "")
                continue;
            if (i === 0) {
                path = (parts[0] === "~") ? home : parts[0];
                result.push({ label: parts[0], path: path });
                continue;
            }
            path += "/" + parts[i];
            result.push({ label: parts[i], path: path });
        }
        return result;
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 10
            spacing: 6

            ToolButton {
                icon.name: "go-up"
                enabled: Browser.canGoUp
                onClicked: Browser.goUp()
                ToolTip.text: qsTr("browser_up")
                ToolTip.visible: hovered
            }

            Flickable {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                clip: true
                contentWidth: crumbs.width
                contentHeight: height
                boundsBehavior: Flickable.StopAtBounds

                Row {
                    id: crumbs
                    height: parent.height
                    spacing: 0

                    Repeater {
                        model: panel.breadcrumbs()

                        delegate: Row {
                            height: crumbs.height
                            spacing: 0

                            Label {
                                text: (index > 0) ? "/" : ""
                                opacity: 0.45
                                font.pixelSize: 13
                                anchors.verticalCenter: parent.verticalCenter
                                color: Kirigami.Theme.textColor
                            }
                            Button {
                                flat: true
                                text: modelData.label
                                font.pixelSize: 13
                                height: crumbs.height
                                onClicked: Browser.setRelativePath(modelData.path)
                            }
                        }
                    }
                }
            }

            Label {
                visible: panel.selectionCount > 0
                text: qsTr("browser_selected").arg(panel.selectionCount)
                font.pixelSize: 12
                opacity: 0.75
                color: Kirigami.Theme.textColor
            }

            ToolButton {
                icon.name: "edit-select-all"
                enabled: Browser.count > 0
                onClicked: panel.selectAll()
                ToolTip.text: qsTr("browser_select_all")
                ToolTip.visible: hovered
            }
            ToolButton {
                icon.name: "edit-clear"
                visible: panel.selectionCount > 0
                onClicked: panel.clearSelection()
                ToolTip.text: qsTr("browser_clear_selection")
                ToolTip.visible: hovered
            }
            Button {
                text: qsTr("action_restore")
                icon.name: "document-revert"
                enabled: panel.selectionCount > 0 && !Restore.running
                onClicked: panel.restoreRequested()
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                           Kirigami.Theme.textColor.b, 0.15)
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: Browser

            delegate: Rectangle {
                id: fileRow

                width: list.width
                height: 38
                color: panel.isSelected(model.relativePath)
                       ? Qt.rgba(Kirigami.Theme.highlightColor.r,
                                 Kirigami.Theme.highlightColor.g,
                                 Kirigami.Theme.highlightColor.b, 0.25)
                       : (rowMouse.containsMouse
                          ? Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                    Kirigami.Theme.textColor.b, 0.07)
                          : "transparent")

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Kirigami.Icon {
                        source: model.isDir ? "folder" : "text-x-generic"
                        implicitWidth: 18
                        implicitHeight: 18
                        Layout.alignment: Qt.AlignVCenter
                    }
                    Label {
                        text: model.name
                        Layout.fillWidth: true
                        elide: Text.ElideMiddle
                        font.pixelSize: 13
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        text: model.modifiedText
                        visible: list.width > 520
                        font.pixelSize: 12
                        opacity: 0.5
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        text: model.sizeText
                        horizontalAlignment: Text.AlignRight
                        Layout.preferredWidth: 80
                        font.pixelSize: 12
                        opacity: 0.65
                        color: Kirigami.Theme.textColor
                    }
                    Kirigami.Icon {
                        visible: panel.isSelected(model.relativePath)
                        source: "checkmark"
                        implicitWidth: 14
                        implicitHeight: 14
                        color: Kirigami.Theme.highlightColor
                        Layout.alignment: Qt.AlignVCenter
                    }
                }

                MouseArea {
                    id: rowMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: (mouse) => {
                        if (mouse.button === Qt.RightButton) {
                            fileMenu.relative = model.relativePath;
                            fileMenu.openAt(fileRow, fileRow.width / 2, fileRow.height / 2);
                        } else {
                            panel.toggle(model.relativePath);
                        }
                    }
                    onDoubleClicked: {
                        if (model.isDir)
                            Browser.setRelativePath(model.relativePath);
                        else
                            App.openPath(Browser.absolutePathFor(model.relativePath));
                    }
                }
            }

            ScrollBar.vertical: ScrollBar {}
        }
    }

    Kirigami.PlaceholderMessage {
        anchors.centerIn: parent
        width: parent.width - 80
        visible: Browser.count === 0
        icon.name: "folder-empty"
        text: qsTr("browser_empty")
        explanation: qsTr("browser_empty_hint")
    }

    AionMenu {
        id: fileMenu
        width: 230

        property string relative: ""

        Column {
            width: parent.width
            AionMenuButton {
                text: qsTr("menu_restore_item")
                onClicked: {
                    panel.selectedPaths = ({ [fileMenu.relative]: true });
                    fileMenu.close();
                    panel.restoreRequested();
                }
            }
            AionMenuButton {
                text: qsTr("menu_reveal_item")
                onClicked: {
                    App.openPath(Browser.absolutePathFor(fileMenu.relative));
                    fileMenu.close();
                }
            }
            AionMenuButton {
                text: qsTr("menu_copy_path")
                onClicked: {
                    App.copyText(Browser.absolutePathFor(fileMenu.relative));
                    fileMenu.close();
                }
            }
        }
    }
}
