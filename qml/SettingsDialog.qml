// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.kde.kirigami as Kirigami

// Settings, styled like Luma's SettingsDialog: an 800x600 overlay card with a
// category list on the left and a scrolling form on the right.
Dialog {
    id: settingsDialog

    parent: Overlay.overlay
    modal: true
    focus: true
    title: qsTr("settings_title")
    padding: 0
    closePolicy: Popup.CloseOnEscape

    width: Math.min(800, (Overlay.overlay ? Overlay.overlay.width : 800) - 32)
    height: Math.min(600, (Overlay.overlay ? Overlay.overlay.height : 600) - 32)
    x: Math.max(12, (Overlay.overlay.width - width) / 2)
    y: Math.max(12, (Overlay.overlay.height - height) / 2)

    readonly property int controlWidth: 240
    readonly property color selectionBackgroundColor: Qt.rgba(Kirigami.Theme.highlightColor.r,
                                                              Kirigami.Theme.highlightColor.g,
                                                              Kirigami.Theme.highlightColor.b, 0.35)
    property string currentCategory: "general"
    property var sources: []
    property var excludes: []
    property var scheduler: ({ state: "absent", error: "" })
    property string pruneMessage: ""

    signal scopeRequested()

    readonly property var categories: [
        { key: "general", text: qsTr("settings_general") },
        { key: "schedule", text: qsTr("settings_schedule") },
        { key: "retention", text: qsTr("settings_retention") }
    ]

    readonly property var intervalModel: {
        const values = [15, 30, 60, 120, 180, 360, 720, 1440];
        return values.map(v => ({
            minutes: v,
            text: v < 60 ? qsTr("schedule_every_minutes").arg(v)
                         : qsTr("schedule_every_hours").arg(v / 60)
        }));
    }

    function refreshLists() {
        sources = Config.sources();
        excludes = Config.excludes();
        scheduler = Scheduler.poll();
    }

    function urlToPath(url) {
        return decodeURIComponent(url.toString().replace(/^file:\/\//, ""));
    }

    function timerStateText() {
        switch (settingsDialog.scheduler.state) {
        case "active": return qsTr("schedule_state_active");
        case "inactive": return qsTr("schedule_state_inactive");
        case "absent": return qsTr("schedule_state_absent");
        }
        return qsTr("schedule_state_unknown");
    }

    onOpened: refreshLists()

    Timer {
        interval: 1200
        repeat: true
        running: settingsDialog.visible
        onTriggered: settingsDialog.scheduler = Scheduler.poll()
    }

    background: Rectangle {
        color: Qt.rgba(Kirigami.Theme.backgroundColor.r,
                       Kirigami.Theme.backgroundColor.g,
                       Kirigami.Theme.backgroundColor.b, 0.97)
        radius: 8
        border.width: 1
        border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                              Kirigami.Theme.textColor.b, 0.15)
    }

    contentItem: RowLayout {
        spacing: 0

        Rectangle {
            Layout.preferredWidth: 220
            Layout.fillHeight: true
            color: "transparent"

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 12
                spacing: 2

                Repeater {
                    model: settingsDialog.categories

                    delegate: Rectangle {
                        Layout.fillWidth: true
                        height: 40
                        radius: 4
                        color: settingsDialog.currentCategory === modelData.key
                               ? settingsDialog.selectionBackgroundColor
                               : (categoryHover.containsMouse
                                  ? Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                            Kirigami.Theme.textColor.b, 0.08)
                                  : "transparent")

                        Rectangle {
                            visible: settingsDialog.currentCategory === modelData.key
                            anchors.left: parent.left
                            anchors.top: parent.top
                            anchors.bottom: parent.bottom
                            anchors.topMargin: 6
                            anchors.bottomMargin: 6
                            width: 3
                            radius: 1.5
                            color: Kirigami.Theme.highlightColor
                        }

                        Label {
                            anchors.left: parent.left
                            anchors.leftMargin: 14
                            anchors.right: parent.right
                            anchors.rightMargin: 8
                            anchors.verticalCenter: parent.verticalCenter
                            text: modelData.text
                            font.pixelSize: 14
                            font.weight: settingsDialog.currentCategory === modelData.key
                                         ? Font.DemiBold : Font.Normal
                            elide: Text.ElideRight
                            color: Kirigami.Theme.textColor
                        }

                        MouseArea {
                            id: categoryHover
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: settingsDialog.currentCategory = modelData.key
                        }
                    }
                }

                Item { Layout.fillHeight: true }

                Button {
                    Layout.fillWidth: true
                    text: qsTr("action_close")
                    onClicked: settingsDialog.close()
                }
            }
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1
            color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                           Kirigami.Theme.textColor.b, 0.15)
        }

        ScrollView {
            id: scroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: availableWidth
            clip: true

            ScrollBar.vertical.policy: ScrollBar.AsNeeded

            ColumnLayout {
                width: scroll.availableWidth - 48
                x: 24
                y: 24
                spacing: 22

                // ============ General ============
                ColumnLayout {
                    visible: settingsDialog.currentCategory === "general"
                    Layout.fillWidth: true
                    spacing: 22

                    Label {
                        text: qsTr("settings_general")
                        font.pixelSize: 20
                        font.weight: Font.Bold
                        color: Kirigami.Theme.textColor
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.2)
                    }

                    GridLayout {
                        columns: 2
                        columnSpacing: 16
                        rowSpacing: 16
                        Layout.fillWidth: true

                        Label {
                            text: qsTr("settings_repo")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            wrapMode: Text.WordWrap
                            color: Kirigami.Theme.textColor
                        }
                        RowLayout {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            spacing: 4

                            TextField {
                                Layout.fillWidth: true
                                font.pixelSize: 13
                                text: Config.repo
                                onEditingFinished: {
                                    if (text !== Config.repo)
                                        Config.setRepo(text);
                                }
                            }
                            ToolButton {
                                icon.name: "folder-open"
                                onClicked: repoDialog.open()
                                ToolTip.text: qsTr("settings_choose_folder")
                                ToolTip.visible: hovered
                            }
                        }

                        Label {
                            text: qsTr("settings_language")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            color: Kirigami.Theme.textColor
                        }
                        Item {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            ComboBox {
                                anchors.fill: parent
                                font.pixelSize: 14
                                model: [
                                    { value: "auto", text: qsTr("language_auto") },
                                    { value: "en", text: "English" },
                                    { value: "ru", text: "Русский" }
                                ]
                                textRole: "text"
                                valueRole: "value"
                                currentIndex: {
                                    const idx = model.findIndex(item => item.value === Config.language);
                                    return idx !== -1 ? idx : 0;
                                }
                                onActivated: (index) => Config.setLanguage(model[index].value)
                            }
                        }

                        Label {
                            text: qsTr("settings_notifications")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            color: Kirigami.Theme.textColor
                        }
                        Item {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            Switch {
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                checked: Config.notifications
                                onToggled: Config.setNotifications(checked)
                            }
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8
                        Button {
                            text: qsTr("settings_scope")
                            icon.name: "folder-saved-search"
                            onClicked: settingsDialog.scopeRequested()
                        }
                        Button {
                            text: qsTr("settings_open_config")
                            icon.name: "text-plain"
                            onClicked: App.openPath(App.configPath())
                        }
                        Item { Layout.fillWidth: true }
                        Button {
                            text: qsTr("settings_reset")
                            icon.name: "edit-undo"
                            onClicked: {
                                Config.reset();
                                settingsDialog.refreshLists();
                            }
                        }
                    }
                }

                // ============ Schedule ============
                ColumnLayout {
                    visible: settingsDialog.currentCategory === "schedule"
                    Layout.fillWidth: true
                    spacing: 22

                    Label {
                        text: qsTr("settings_schedule")
                        font.pixelSize: 20
                        font.weight: Font.Bold
                        color: Kirigami.Theme.textColor
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.2)
                    }

                    GridLayout {
                        columns: 2
                        columnSpacing: 16
                        rowSpacing: 16
                        Layout.fillWidth: true

                        Label {
                            text: qsTr("settings_schedule_enabled")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            wrapMode: Text.WordWrap
                            color: Kirigami.Theme.textColor
                        }
                        Item {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            Switch {
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                checked: Config.scheduleEnabled
                                onToggled: Config.setScheduleEnabled(checked)
                            }
                        }

                        Label {
                            text: qsTr("settings_schedule_interval")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            color: Kirigami.Theme.textColor
                        }
                        Item {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            ComboBox {
                                anchors.fill: parent
                                font.pixelSize: 14
                                model: settingsDialog.intervalModel
                                textRole: "text"
                                valueRole: "minutes"
                                currentIndex: {
                                    const idx = model.findIndex(entry => entry.minutes === Config.scheduleMinutes);
                                    return idx !== -1 ? idx : 2;
                                }
                                onActivated: (index) => Config.setScheduleMinutes(model[index].minutes)
                            }
                        }
                    }

                    Label {
                        Layout.fillWidth: true
                        text: qsTr("settings_schedule_state").arg(settingsDialog.timerStateText())
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.75
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        visible: settingsDialog.scheduler.error !== undefined
                                 && settingsDialog.scheduler.error !== ""
                        text: settingsDialog.scheduler.error !== undefined
                              ? settingsDialog.scheduler.error : ""
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        color: Kirigami.Theme.negativeTextColor
                    }
                }

                // ============ Retention ============
                ColumnLayout {
                    visible: settingsDialog.currentCategory === "retention"
                    Layout.fillWidth: true
                    spacing: 22

                    Label {
                        text: qsTr("settings_retention")
                        font.pixelSize: 20
                        font.weight: Font.Bold
                        color: Kirigami.Theme.textColor
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.2)
                    }
                    Label {
                        Layout.fillWidth: true
                        text: qsTr("settings_retention_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }

                    GridLayout {
                        columns: 2
                        columnSpacing: 16
                        rowSpacing: 16
                        Layout.fillWidth: true

                        Label {
                            text: qsTr("settings_keep_hourly")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            color: Kirigami.Theme.textColor
                        }
                        Item {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            SpinBox {
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                from: 1
                                to: 720
                                value: Config.keepHourly
                                onValueModified: Config.setKeepHourly(value)
                            }
                        }

                        Label {
                            text: qsTr("settings_keep_daily")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            color: Kirigami.Theme.textColor
                        }
                        Item {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            SpinBox {
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                from: 0
                                to: 3650
                                value: Config.keepDaily
                                onValueModified: Config.setKeepDaily(value)
                            }
                        }

                        Label {
                            text: qsTr("settings_keep_weekly")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            color: Kirigami.Theme.textColor
                        }
                        Item {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            SpinBox {
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                from: 0
                                to: 520
                                value: Config.keepWeekly
                                onValueModified: Config.setKeepWeekly(value)
                            }
                        }
                    }

                    Button {
                        text: qsTr("settings_prune_now")
                        icon.name: "edit-clear-history"
                        onClicked: {
                            const ok = Store.pruneNow() > 0;
                            settingsDialog.pruneMessage = ok
                                ? qsTr("settings_pruned_now")
                                : qsTr("settings_nothing_to_prune");
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        visible: settingsDialog.pruneMessage !== ""
                        text: settingsDialog.pruneMessage
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.75
                        color: Kirigami.Theme.textColor
                    }
                }
            }
        }
    }

    FolderDialog {
        id: repoDialog
        title: qsTr("settings_choose_folder")
        onAccepted: Config.setRepo(settingsDialog.urlToPath(selectedFolder))
    }
    FolderDialog {
        id: sourceFolderDialog
        title: qsTr("settings_add_folder")
        onAccepted: {
            Config.addSource(settingsDialog.urlToPath(selectedFolder));
            settingsDialog.refreshLists();
        }
    }
    FileDialog {
        id: sourceFileDialog
        title: qsTr("settings_add_file")
        fileMode: FileDialog.OpenFile
        onAccepted: {
            Config.addSource(settingsDialog.urlToPath(selectedFile));
            settingsDialog.refreshLists();
        }
    }
    FolderDialog {
        id: excludeFolderDialog
        title: qsTr("settings_add_excluded_folder")
        onAccepted: {
            Config.addExclude(settingsDialog.urlToPath(selectedFolder));
            settingsDialog.refreshLists();
        }
    }
}
