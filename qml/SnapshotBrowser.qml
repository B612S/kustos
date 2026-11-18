// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// File browser for one snapshot: breadcrumbs, a list with selection and the
// restore entry point. Contents come from the Rust `Browser` object, which
// keeps the snapshot listing in memory.
Rectangle {
    id: panel

    color: Kirigami.Theme.backgroundColor

    property var entries: []
    property var crumbs: []
    property string displayPath: "/"
    property bool canUp: false
    property string snapshotDate: ""
    property string snapshotTime: ""
    property bool snapshotLatest: false
    property bool hasSnapshot: false
    property bool busy: false
    property var selectedPaths: ({})
    readonly property int selectionCount: Object.keys(selectedPaths).length

    signal restoreRequested()

    function loadSnapshot(id) {
        Browser.setSnapshot(id);
        hasSnapshot = id !== "";
        refresh();
    }

    function refresh() {
        entries = Browser.entries();
        displayPath = Browser.displayPath();
        canUp = Browser.canGoUp();
        crumbs = buildBreadcrumbs();
    }

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
        for (let i = 0; i < entries.length; ++i)
            copy[entries[i].relativePath] = true;
        selectedPaths = copy;
    }

    function clearSelection() {
        selectedPaths = ({});
    }

    function enterRow(row) {
        if (row < 0 || row >= entries.length)
            return;
        if (!entries[row].isDir) {
            App.openPath(Store.repoPath());
            return;
        }
        Browser.setPath(entries[row].relativePath);
        refresh();
    }

    function goUp() {
        Browser.goUp();
        refresh();
    }

    function buildBreadcrumbs() {
        const relative = Browser.path();
        const home = Browser.homeRoot();
        const result = [];
        let base = "";
        if (home !== "" && (relative === home || relative.startsWith(home + "/"))) {
            result.push({ label: qsTr("browser_home"), path: home });
            base = home;
        }
        const rest = relative.slice(base.length).split("/");
        let path = base;
        for (let i = 0; i < rest.length; ++i) {
            if (rest[i] === "")
                continue;
            path += (path === "" ? "" : "/") + rest[i];
            result.push({ label: rest[i], path: path });
        }
        if (result.length === 0)
            result.push({ label: "/", path: "" });
        return result;
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 12
            Layout.rightMargin: 12
            Layout.topMargin: 10
            spacing: 8
            visible: panel.snapshotDate !== ""

            Label {
                text: qsTr("browser_snapshot").arg(panel.snapshotDate).arg(panel.snapshotTime)
                font.pixelSize: 14
                font.weight: Font.DemiBold
                color: Kirigami.Theme.textColor
            }
            Rectangle {
                visible: panel.snapshotLatest
                radius: 4
                color: Qt.rgba(Kirigami.Theme.highlightColor.r,
                               Kirigami.Theme.highlightColor.g,
                               Kirigami.Theme.highlightColor.b, 0.25)
                implicitWidth: headerLatest.implicitWidth + 12
                implicitHeight: headerLatest.implicitHeight + 4
                Label {
                    id: headerLatest
                    anchors.centerIn: parent
                    text: qsTr("snapshot_latest")
                    font.pixelSize: 11
                    color: Kirigami.Theme.textColor
                }
            }
            Item { Layout.fillWidth: true }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 10
            Layout.rightMargin: 10
            Layout.bottomMargin: 6
            spacing: 6

            ToolButton {
                icon.name: "go-up"
                enabled: panel.canUp
                onClicked: panel.goUp()
                ToolTip.text: qsTr("browser_up")
                ToolTip.visible: hovered
            }

            Flickable {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                clip: true
                contentWidth: crumbRow.width
                contentHeight: height
                boundsBehavior: Flickable.StopAtBounds

                Row {
                    id: crumbRow
                    height: parent.height
                    spacing: 0

                    Repeater {
                        model: panel.crumbs

                        delegate: Row {
                            height: crumbRow.height
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
                                height: crumbRow.height
                                onClicked: {
                                    Browser.setPath(modelData.path);
                                    panel.refresh();
                                }
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
                enabled: panel.entries.length > 0
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
                enabled: panel.selectionCount > 0 && !panel.busy
                onClicked: panel.restoreRequested()
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                           Kirigami.Theme.textColor.b, 0.15)
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 10
            Layout.rightMargin: 10
            Layout.bottomMargin: 2
            spacing: 8

            Label {
                text: qsTr("browser_name")
                Layout.fillWidth: true
                Layout.leftMargin: 26
                font.pixelSize: 11
                opacity: 0.5
                color: Kirigami.Theme.textColor
            }
            Label {
                text: qsTr("browser_modified")
                visible: list.width > 520
                Layout.preferredWidth: 140
                font.pixelSize: 11
                opacity: 0.5
                color: Kirigami.Theme.textColor
            }
            Label {
                text: qsTr("browser_size")
                Layout.preferredWidth: 80
                horizontalAlignment: Text.AlignRight
                font.pixelSize: 11
                opacity: 0.5
                color: Kirigami.Theme.textColor
            }
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: panel.entries

            delegate: Rectangle {
                id: fileRow

                width: list.width
                height: 38
                color: panel.isSelected(modelData.relativePath)
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
                        source: modelData.isDir ? "folder" : "text-x-generic"
                        implicitWidth: 18
                        implicitHeight: 18
                        Layout.alignment: Qt.AlignVCenter
                    }
                    Label {
                        text: modelData.name
                        Layout.fillWidth: true
                        elide: Text.ElideMiddle
                        font.pixelSize: 13
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        text: modelData.modifiedText
                        visible: list.width > 520
                        Layout.preferredWidth: 140
                        font.pixelSize: 12
                        opacity: 0.5
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        text: modelData.isDir ? "" : modelData.sizeText
                        horizontalAlignment: Text.AlignRight
                        Layout.preferredWidth: 80
                        font.pixelSize: 12
                        opacity: 0.65
                        color: Kirigami.Theme.textColor
                    }
                    Kirigami.Icon {
                        visible: panel.isSelected(modelData.relativePath)
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
                            fileMenu.relative = modelData.relativePath;
                            fileMenu.openAt(fileRow, fileRow.width / 2, fileRow.height / 2);
                        } else {
                            panel.toggle(modelData.relativePath);
                        }
                    }
                    onDoubleClicked: {
                        if (modelData.isDir) {
                            Browser.setPath(modelData.relativePath);
                            panel.refresh();
                        }
                    }
                }
            }

            ScrollBar.vertical: ScrollBar {}
        }
    }

    Kirigami.PlaceholderMessage {
        anchors.centerIn: parent
        width: parent.width - 80
        visible: panel.entries.length === 0 && panel.hasSnapshot
        icon.name: "folder-empty"
        text: qsTr("browser_empty")
        explanation: qsTr("browser_empty_hint")
    }

    KustosMenu {
        id: fileMenu
        width: 230

        property string relative: ""

        Column {
            width: parent.width
            KustosMenuButton {
                text: qsTr("menu_restore_item")
                onClicked: {
                    panel.selectedPaths = ({ [fileMenu.relative]: true });
                    fileMenu.close();
                    panel.restoreRequested();
                }
            }
            KustosMenuButton {
                text: qsTr("menu_copy_path")
                onClicked: {
                    App.copyText(fileMenu.relative);
                    fileMenu.close();
                }
            }
        }
    }
}
