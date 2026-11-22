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
    z: 2000

    width: Math.min(800, (Overlay.overlay ? Overlay.overlay.width : 800) - 32)
    height: Math.min(600, (Overlay.overlay ? Overlay.overlay.height : 600) - 32)
    x: Math.max(12, (Overlay.overlay.width - width) / 2)
    y: Math.max(12, (Overlay.overlay.height - height) / 2)

    readonly property int controlWidth: 240
    readonly property color selectionBackgroundColor: Qt.rgba(Kirigami.Theme.highlightColor.r,
                                                              Kirigami.Theme.highlightColor.g,
                                                              Kirigami.Theme.highlightColor.b, 0.35)
    property string currentCategory: "general"
    property string pruneMessage: ""
    property var prunePlan: []
    property var checkState: ({ running: false, ok: false, error: "", detail: "",
                                lastCheck: "", lastCheckOk: false, lastCheckDetail: "",
                                stale: true, hasRepo: false })
    property string recoveryKey: ""
    property string recoveryMessage: ""

    signal rulesRequested()
    signal commonPasswordRequested()

    readonly property var categories: [
        { key: "general", text: qsTr("settings_general") },
        { key: "security", text: qsTr("settings_security") },
        { key: "retention", text: qsTr("settings_retention") },
        { key: "maintenance", text: qsTr("settings_maintenance") }
    ]

    function urlToPath(url) {
        return decodeURIComponent(url.toString().replace(/^file:\/\//, ""));
    }

    function refreshCheck() {
        checkState = Maintenance.status();
    }

    function checkSummary() {
        if (checkState.running)
            return qsTr("settings_check_running");
        if (!checkState.hasRepo)
            return qsTr("settings_check_no_repo");
        if (checkState.lastCheck === "")
            return qsTr("settings_check_never");
        if (checkState.stale)
            return qsTr("settings_check_stale").arg(checkState.lastCheck);
        return (checkState.lastCheckOk ? qsTr("settings_check_ok")
                                       : qsTr("settings_check_failed")).arg(checkState.lastCheck);
    }

    function requestPrune() {
        const preview = Store.prunePreview();
        if (preview.error !== "") {
            pruneMessage = preview.error;
            return;
        }
        prunePlan = preview.items;
        if (prunePlan.length === 0) {
            pruneMessage = qsTr("settings_nothing_to_prune");
            return;
        }
        pruneMessage = "";
        pruneDialog.open();
    }

    function confirmPrune() {
        const ok = Store.pruneNow() > 0;
        pruneDialog.close();
        pruneMessage = ok ? qsTr("settings_pruned_now")
                          : qsTr("settings_prune_failed");
        if (ok)
            Store.refreshSnapshots();
    }

    function toggleRecoveryKey() {
        if (recoveryKey !== "") {
            recoveryKey = "";
            return;
        }
        const key = Store.recoveryKey();
        if (key === "") {
            recoveryMessage = qsTr("settings_recovery_missing");
            return;
        }
        recoveryMessage = "";
        recoveryKey = key;
    }

    onOpened: refreshCheck()
    onClosed: {
        recoveryKey = "";
        recoveryMessage = "";
    }

    onCurrentCategoryChanged: {
        if (currentCategory === "maintenance") {
            refreshCheck();
        } else {
            recoveryKey = "";
            recoveryMessage = "";
        }
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

                    Flow {
                        Layout.fillWidth: true
                        spacing: 8
                        Button {
                            text: qsTr("settings_rules")
                            icon.name: "document-properties"
                            onClicked: settingsDialog.rulesRequested()
                        }
                        Button {
                            text: qsTr("settings_open_config")
                            icon.name: "text-plain"
                            onClicked: App.openPath(App.configPath())
                        }
                        Button {
                            text: qsTr("settings_reset")
                            icon.name: "edit-undo"
                            onClicked: Config.reset()
                        }
                    }
                }

                // ============ Security ============
                ColumnLayout {
                    visible: settingsDialog.currentCategory === "security"
                    Layout.fillWidth: true
                    spacing: 22

                    Label {
                        text: qsTr("settings_security")
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
                            text: qsTr("settings_password_mode")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            wrapMode: Text.WordWrap
                            color: Kirigami.Theme.textColor
                        }
                        Item {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            ComboBox {
                                anchors.fill: parent
                                font.pixelSize: 14
                                model: [
                                    { value: "per_rule", text: qsTr("password_mode_per_rule") },
                                    { value: "common", text: qsTr("password_mode_common") }
                                ]
                                textRole: "text"
                                valueRole: "value"
                                currentIndex: {
                                    const idx = model.findIndex(item => item.value === Config.passwordMode);
                                    return idx !== -1 ? idx : 0;
                                }
                                onActivated: (index) => {
                                    const value = model[index].value;
                                    Config.setPasswordMode(value);
                                    if (value === "common" && !Config.commonPasswordSet)
                                        settingsDialog.commonPasswordRequested();
                                }
                            }
                        }

                        Label {
                            visible: Config.passwordMode === "common"
                            text: qsTr("settings_common_password")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            wrapMode: Text.WordWrap
                            color: Kirigami.Theme.textColor
                        }
                        ColumnLayout {
                            visible: Config.passwordMode === "common"
                            Layout.preferredWidth: settingsDialog.controlWidth
                            spacing: 4
                            Label {
                                Layout.fillWidth: true
                                text: Config.commonPasswordSet
                                      ? qsTr("settings_common_password_set")
                                      : qsTr("settings_common_password_missing")
                                font.pixelSize: 12
                                wrapMode: Text.WordWrap
                                color: Config.commonPasswordSet ? Kirigami.Theme.textColor
                                                                : Kirigami.Theme.negativeTextColor
                            }
                            Button {
                                Layout.fillWidth: true
                                text: Config.commonPasswordSet ? qsTr("action_change_password")
                                                               : qsTr("action_set_password")
                                icon.name: "dialog-password"
                                onClicked: settingsDialog.commonPasswordRequested()
                            }
                        }
                    }

                    Label {
                        Layout.fillWidth: true
                        text: qsTr("settings_password_mode_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        visible: Config.passwordMode === "common"
                        text: qsTr("settings_common_password_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.2)
                        height: 1
                    }

                    Label {
                        Layout.fillWidth: true
                        text: qsTr("settings_recovery_title")
                        font.pixelSize: 15
                        font.weight: Font.DemiBold
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        text: qsTr("settings_recovery_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8
                        Button {
                            text: settingsDialog.recoveryKey === ""
                                  ? qsTr("settings_recovery_show") : qsTr("settings_recovery_hide")
                            icon.name: settingsDialog.recoveryKey === "" ? "dialog-password" : "view-hidden"
                            onClicked: settingsDialog.toggleRecoveryKey()
                        }
                        Button {
                            visible: settingsDialog.recoveryKey !== ""
                            text: qsTr("action_copy_key")
                            icon.name: "edit-copy"
                            onClicked: App.copyText(settingsDialog.recoveryKey)
                        }
                    }
                    TextField {
                        Layout.fillWidth: true
                        visible: settingsDialog.recoveryKey !== ""
                        readOnly: true
                        selectByMouse: true
                        font.family: "monospace"
                        text: settingsDialog.recoveryKey
                    }
                    Label {
                        Layout.fillWidth: true
                        visible: settingsDialog.recoveryKey !== ""
                        text: qsTr("settings_recovery_warning")
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        color: Kirigami.Theme.negativeTextColor
                    }
                    Label {
                        Layout.fillWidth: true
                        visible: settingsDialog.recoveryMessage !== ""
                        text: settingsDialog.recoveryMessage
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        color: Kirigami.Theme.negativeTextColor
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.2)
                        height: 1
                    }

                    GridLayout {
                        columns: 2
                        columnSpacing: 16
                        rowSpacing: 16
                        Layout.fillWidth: true

                        Label {
                            text: qsTr("settings_lock_timeout")
                            font.pixelSize: 14
                            Layout.fillWidth: true
                            Layout.preferredWidth: Math.max(120,
                                settingsDialog.width - 220 - 48
                                - settingsDialog.controlWidth - 16 - 2)
                            Layout.maximumWidth: Math.max(120,
                                settingsDialog.width - 220 - 48
                                - settingsDialog.controlWidth - 16 - 2)
                            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
                            wrapMode: Text.WordWrap
                            color: Kirigami.Theme.textColor
                        }
                        Item {
                            Layout.preferredWidth: settingsDialog.controlWidth
                            Layout.preferredHeight: 40
                            ComboBox {
                                anchors.fill: parent
                                font.pixelSize: 14
                                model: [
                                    { value: 0, text: qsTr("lock_never") },
                                    { value: 1, text: qsTr("lock_minutes").arg(1) },
                                    { value: 5, text: qsTr("lock_minutes").arg(5) },
                                    { value: 15, text: qsTr("lock_minutes").arg(15) },
                                    { value: 30, text: qsTr("lock_minutes").arg(30) },
                                    { value: 60, text: qsTr("lock_minutes").arg(60) },
                                    { value: 120, text: qsTr("lock_minutes").arg(120) },
                                    { value: 240, text: qsTr("lock_minutes").arg(240) }
                                ]
                                textRole: "text"
                                valueRole: "value"
                                currentIndex: {
                                    const idx = model.findIndex(item => item.value === Config.lockTimeoutMinutes);
                                    return idx !== -1 ? idx : 3;
                                }
                                onActivated: (index) => Config.setLockTimeoutMinutes(model[index].value)
                            }
                        }
                    }

                    Label {
                        Layout.fillWidth: true
                        text: qsTr("settings_lock_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }

                    Flow {
                        Layout.fillWidth: true
                        spacing: 8
                        Button {
                            text: qsTr("action_lock_now")
                            icon.name: "object-locked"
                            onClicked: Store.lockNow()
                        }
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
                        onClicked: settingsDialog.requestPrune()
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

                // ============ Maintenance ============
                ColumnLayout {
                    visible: settingsDialog.currentCategory === "maintenance"
                    Layout.fillWidth: true
                    spacing: 14

                    Label {
                        text: qsTr("settings_maintenance")
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
                        text: qsTr("settings_check_title")
                        font.pixelSize: 15
                        font.weight: Font.DemiBold
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        text: qsTr("settings_check_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        text: settingsDialog.checkSummary()
                        font.pixelSize: 13
                        wrapMode: Text.WordWrap
                        color: (!settingsDialog.checkState.hasRepo
                                || settingsDialog.checkState.error !== ""
                                || settingsDialog.checkState.stale
                                || (!settingsDialog.checkState.running
                                    && settingsDialog.checkState.lastCheck !== ""
                                    && !settingsDialog.checkState.lastCheckOk))
                               ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        visible: !settingsDialog.checkState.running
                                 && settingsDialog.checkState.lastCheckDetail !== ""
                                 && !settingsDialog.checkState.lastCheckOk
                        text: settingsDialog.checkState.lastCheckDetail
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        opacity: 0.75
                        color: Kirigami.Theme.negativeTextColor
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        Button {
                            text: qsTr("settings_check_now")
                            icon.name: "view-refresh"
                            enabled: settingsDialog.checkState.hasRepo
                                     && !settingsDialog.checkState.running
                            onClicked: {
                                Maintenance.start(false);
                                settingsDialog.refreshCheck();
                            }
                        }
                        Button {
                            text: qsTr("settings_check_data")
                            icon.name: "media-playback-start"
                            enabled: settingsDialog.checkState.hasRepo
                                     && !settingsDialog.checkState.running
                            onClicked: {
                                Maintenance.start(true);
                                settingsDialog.refreshCheck();
                            }
                        }
                        BusyIndicator {
                            running: settingsDialog.checkState.running
                            visible: running
                            implicitWidth: 28
                            implicitHeight: 28
                        }
                    }
                }
            }
        }
    }

    Timer {
        interval: 700
        repeat: true
        running: settingsDialog.opened && settingsDialog.currentCategory === "maintenance"
        onTriggered: settingsDialog.refreshCheck()
    }

    Dialog {
        id: pruneDialog
        parent: Overlay.overlay
        modal: true
        focus: true
        padding: 20
        closePolicy: Popup.CloseOnEscape
        width: Math.min(520, (Overlay.overlay ? Overlay.overlay.width : 960) - 48)
        height: Math.min(460, (Overlay.overlay ? Overlay.overlay.height : 640) - 48)
        x: Math.max(12, (Overlay.overlay.width - width) / 2)
        y: Math.max(12, (Overlay.overlay.height - height) / 2)

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
            spacing: 12

            Label {
                Layout.fillWidth: true
                text: qsTr("settings_prune_title").arg(settingsDialog.prunePlan.length)
                font.pixelSize: 18
                font.weight: Font.DemiBold
                color: Kirigami.Theme.textColor
            }
            Label {
                Layout.fillWidth: true
                text: qsTr("settings_prune_message")
                wrapMode: Text.WordWrap
                font.pixelSize: 12
                opacity: 0.75
                color: Kirigami.Theme.textColor
            }
            ListView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: settingsDialog.prunePlan
                ScrollBar.vertical: ScrollBar {}

                delegate: RowLayout {
                    width: ListView.view.width
                    height: 28
                    spacing: 8

                    Label {
                        text: modelData.dateText + " " + modelData.timeText
                        font.pixelSize: 12
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        text: modelData.ruleName !== ""
                              ? modelData.ruleName + " · " + modelData.shortId
                              : modelData.shortId
                        elide: Text.ElideRight
                        font.pixelSize: 12
                        opacity: 0.6
                        color: Kirigami.Theme.textColor
                    }
                }
            }
            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                Item { Layout.fillWidth: true }
                Button {
                    text: qsTr("action_cancel")
                    onClicked: pruneDialog.close()
                }
                Button {
                    text: qsTr("settings_prune_confirm")
                    icon.name: "edit-delete"
                    onClicked: settingsDialog.confirmPrune()
                }
            }
        }
    }

    FolderDialog {
        id: repoDialog
        title: qsTr("settings_choose_folder")
        onAccepted: Config.setRepo(settingsDialog.urlToPath(selectedFolder))
    }
}
