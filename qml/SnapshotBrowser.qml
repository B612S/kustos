// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
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
    property bool menuEnabled: true
    property bool loading: false
    property string loadError: ""
    property string snapshotId: ""
    property var selectedPaths: ({})
    property bool searchOpen: false
    property string searchQuery: ""
    property var searchResults: []
    readonly property bool searching: searchOpen && searchQuery.trim().length >= 2
    readonly property int selectionCount: Object.keys(selectedPaths).length
    readonly property bool canOpenSelection: {
        const keys = Object.keys(selectedPaths);
        if (keys.length !== 1)
            return false;
        for (let i = 0; i < entries.length; ++i) {
            if (entries[i].relativePath === keys[0])
                return !entries[i].isDir;
        }
        return false;
    }

    signal restoreRequested()
    signal exportRequested(string destination)
    signal openRequested(string relative)
    signal loadFailed(string message)

    onMenuEnabledChanged: {
        if (!menuEnabled)
            fileMenu.close();
    }

    function loadSnapshot(id) {
        closeSearch();
        snapshotId = id;
        clearSelection();
        Browser.setSnapshot(id);
        hasSnapshot = id !== "";
        loadError = "";
        loading = Browser.loading();
        refresh();
        if (loading)
            loadTimer.start();
        else
            loadTimer.stop();
    }

    function pollLoad() {
        const state = Browser.poll();
        if (!state.done)
            return;
        loading = false;
        loadTimer.stop();
        loadError = state.error;
        refresh();
        if (loadError !== "")
            panel.loadFailed(loadError);
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

    // A plain click behaves like Dolphin: it selects that item and clears the
    // other rows, but clicking an already selected single row keeps it.
    function selectOnly(relative) {
        if (selectionCount === 1 && selectedPaths[relative] === true)
            return;
        selectedPaths = ({ [relative]: true });
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

    function openSearch() {
        if (!hasSnapshot || loading)
            return;
        searchOpen = true;
        searchQuery = "";
        searchResults = [];
        searchField.forceActiveFocus();
    }

    function closeSearch() {
        searchOpen = false;
        searchQuery = "";
        searchResults = [];
    }

    function runSearch() {
        if (!searchOpen || searchQuery.trim().length < 2) {
            searchResults = [];
            return;
        }
        searchResults = Browser.search(searchQuery);
    }

    function searchSummary() {
        if (searchQuery.trim().length < 2)
            return qsTr("browser_search_hint");
        return searchResults.length === 0
                ? qsTr("browser_search_none")
                : qsTr("browser_search_count").arg(searchResults.length);
    }

    function openSearchResult(hit) {
        const target = hit.isDir ? hit.relativePath : hit.parent;
        closeSearch();
        navigate(target);
        if (!hit.isDir)
            selectOnly(hit.relativePath);
    }

    function navigate(path) {
        Browser.setPath(path);
        clearSelection();
        refresh();
    }

    function openItem(relative) {
        if (relative === "" || snapshotId === "" || busy)
            return;
        openRequested(relative);
    }

    function openSelection() {
        if (!canOpenSelection)
            return;
        openItem(Object.keys(selectedPaths)[0]);
    }

    // "Save to Folder…": pick a folder in the file manager and copy the
    // selected items there directly, without restoring the original tree.
    function saveSelected() {
        if (selectionCount === 0 || busy || snapshotId === "")
            return;
        exportFolderDialog.open();
    }

    function saveItem(relative) {
        selectedPaths = ({ [relative]: true });
        saveSelected();
    }

    function urlToPath(url) {
        return decodeURIComponent(url.toString().replace(/^file:\/\//, ""));
    }

    function goUp() {
        Browser.goUp();
        clearSelection();
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
                Layout.fillWidth: true
                text: qsTr("browser_snapshot").arg(panel.snapshotDate).arg(panel.snapshotTime)
                font.pixelSize: 14
                font.weight: Font.DemiBold
                elide: Text.ElideRight
                color: Kirigami.Theme.textColor
            }
            Rectangle {
                visible: panel.snapshotLatest && panel.width > 330
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
                                    panel.navigate(modelData.path);
                                }
                            }
                        }
                    }
                }
            }

            Label {
                visible: panel.selectionCount > 0 && panel.width > 500
                text: qsTr("browser_selected").arg(panel.selectionCount)
                font.pixelSize: 12
                opacity: 0.75
                color: Kirigami.Theme.textColor
            }

            ToolButton {
                icon.name: "search"
                enabled: panel.hasSnapshot && !panel.loading
                onClicked: panel.searchOpen ? panel.closeSearch() : panel.openSearch()
                ToolTip.text: qsTr("browser_search")
                ToolTip.visible: hovered
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
            ToolButton {
                icon.name: "document-open"
                enabled: panel.canOpenSelection && !panel.busy
                onClicked: panel.openSelection()
                ToolTip.text: qsTr("menu_open_item")
                ToolTip.visible: hovered
            }
            ToolButton {
                icon.name: "document-save-as"
                enabled: panel.selectionCount > 0 && !panel.busy && panel.snapshotId !== ""
                onClicked: panel.saveSelected()
                ToolTip.text: qsTr("action_save_copies")
                ToolTip.visible: hovered
            }
            ToolButton {
                icon.name: "document-revert"
                enabled: panel.selectionCount > 0 && !panel.busy
                onClicked: panel.restoreRequested()
                ToolTip.text: qsTr("action_restore")
                ToolTip.visible: hovered
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 10
            Layout.rightMargin: 10
            Layout.bottomMargin: 4
            spacing: 6
            visible: panel.searchOpen

            TextField {
                id: searchField
                Layout.fillWidth: true
                font.pixelSize: 13
                selectByMouse: true
                text: panel.searchQuery
                placeholderText: qsTr("browser_search_placeholder")
                onTextChanged: {
                    panel.searchQuery = text;
                    searchTimer.restart();
                }
                Keys.onEscapePressed: panel.closeSearch()
            }
            Label {
                text: panel.searchSummary()
                font.pixelSize: 11
                opacity: 0.6
                elide: Text.ElideRight
                Layout.maximumWidth: 130
                color: Kirigami.Theme.textColor
            }
            ToolButton {
                icon.name: "window-close"
                onClicked: panel.closeSearch()
                ToolTip.text: qsTr("browser_search_clear")
                ToolTip.visible: hovered
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
            Layout.rightMargin: 42
            Layout.bottomMargin: 2
            spacing: 8
            visible: !panel.searching

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
            // Reserve the checkmark column so selecting a row never shifts
            // the Name/Modified/Size columns.
            Item {
                Layout.preferredWidth: 14
                Layout.preferredHeight: 1
            }
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            visible: !panel.searching
            model: panel.entries

            delegate: Rectangle {
                id: fileRow

                // Leave the overlay scrollbar its own gutter: the selection
                // highlight must not run underneath it.
                width: list.width - 18
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
                    anchors.rightMargin: 24
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
                    Item {
                        Layout.preferredWidth: 14
                        Layout.preferredHeight: 14
                        Layout.alignment: Qt.AlignVCenter
                        Kirigami.Icon {
                            anchors.centerIn: parent
                            visible: panel.isSelected(modelData.relativePath)
                            source: "checkmark"
                            implicitWidth: 14
                            implicitHeight: 14
                            color: Kirigami.Theme.highlightColor
                        }
                    }
                }

                MouseArea {
                    id: rowMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: (mouse) => {
                        if (mouse.button === Qt.RightButton) {
                            if (!panel.menuEnabled)
                                return;
                            fileMenu.relative = modelData.relativePath;
                            fileMenu.isDir = modelData.isDir;
                            fileMenu.openAt(fileRow, mouse.x, mouse.y);
                            return;
                        }
                        if (mouse.modifiers & Qt.ControlModifier)
                            panel.toggle(modelData.relativePath);
                        else
                            panel.selectOnly(modelData.relativePath);
                    }
                    onDoubleClicked: {
                        if (modelData.isDir)
                            panel.navigate(modelData.relativePath);
                        else
                            panel.openItem(modelData.relativePath);
                    }
                }
            }

            ScrollBar.vertical: ScrollBar {}
        }

        ListView {
            id: searchList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            visible: panel.searching
            model: panel.searchResults
            ScrollBar.vertical: ScrollBar {}

            delegate: Rectangle {
                width: searchList.width - 18
                height: 44
                color: resultMouse.containsMouse
                       ? Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                 Kirigami.Theme.textColor.b, 0.07)
                       : "transparent"

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 24
                    spacing: 8

                    Kirigami.Icon {
                        source: modelData.isDir ? "folder" : "text-x-generic"
                        implicitWidth: 18
                        implicitHeight: 18
                        Layout.alignment: Qt.AlignVCenter
                    }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 1

                        Label {
                            Layout.fillWidth: true
                            text: modelData.name
                            elide: Text.ElideMiddle
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }
                        Label {
                            Layout.fillWidth: true
                            text: modelData.relativePath
                            elide: Text.ElideMiddle
                            font.pixelSize: 10
                            opacity: 0.5
                            color: Kirigami.Theme.textColor
                        }
                    }
                    Label {
                        text: modelData.isDir ? "" : modelData.sizeText
                        horizontalAlignment: Text.AlignRight
                        Layout.preferredWidth: 80
                        font.pixelSize: 12
                        opacity: 0.65
                        color: Kirigami.Theme.textColor
                    }
                }

                MouseArea {
                    id: resultMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: panel.openSearchResult(modelData)
                }
            }
        }
    }

    ColumnLayout {
        anchors.centerIn: parent
        width: Math.min(360, parent.width - 80)
        spacing: 10
        visible: panel.loading
        z: 2

        BusyIndicator {
            running: panel.loading
            Layout.alignment: Qt.AlignHCenter
        }
        Label {
            Layout.fillWidth: true
            text: qsTr("browser_loading")
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            font.pixelSize: 13
            color: Kirigami.Theme.textColor
        }
    }

    Timer {
        id: loadTimer
        interval: 100
        repeat: true
        onTriggered: panel.pollLoad()
    }

    Timer {
        id: searchTimer
        interval: 180
        onTriggered: panel.runSearch()
    }

    Shortcut {
        sequence: "Ctrl+F"
        onActivated: panel.searchOpen ? panel.closeSearch() : panel.openSearch()
    }

    Kirigami.PlaceholderMessage {
        anchors.centerIn: parent
        width: parent.width - 60
        visible: !panel.hasSnapshot && !panel.loading
        icon.name: "document-open-folder"
        text: qsTr("browser_no_snapshot")
        explanation: qsTr("browser_no_snapshot_hint")
    }

    Kirigami.PlaceholderMessage {
        anchors.centerIn: parent
        width: parent.width - 80
        visible: panel.entries.length === 0 && panel.hasSnapshot && !panel.loading
        icon.name: "folder-empty"
        text: qsTr("browser_empty")
        explanation: qsTr("browser_empty_hint")
    }

    FolderDialog {
        id: exportFolderDialog
        title: qsTr("browser_choose_export_folder")
        onAccepted: {
            const destination = panel.urlToPath(selectedFolder);
            if (destination !== "" && panel.selectionCount > 0 && panel.snapshotId !== "")
                panel.exportRequested(destination);
        }
    }

    KustosMenu {
        id: fileMenu
        width: 230

        property string relative: ""
        property bool isDir: false

        Column {
            width: parent.width
            KustosMenuButton {
                text: qsTr("menu_open_item")
                visible: !fileMenu.isDir
                onClicked: {
                    const relative = fileMenu.relative;
                    fileMenu.close();
                    panel.openItem(relative);
                }
            }
            KustosMenuButton {
                text: qsTr("menu_save_item")
                onClicked: {
                    const relative = fileMenu.relative;
                    fileMenu.close();
                    panel.saveItem(relative);
                }
            }
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
