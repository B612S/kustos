// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.kde.kirigami as Kirigami

// Editor for one backup rule: name, sources, exclusions and schedule. The
// rule is chosen in the main window's left column (or created there), so this
// dialog does not repeat the rule list. Every change is saved immediately, so
// the next backup (manual or scheduled) uses it. Buttons that add something
// live in the header of the section they add to.
Dialog {
    id: dialog

    parent: Overlay.overlay
    modal: true
    focus: true
    title: qsTr("backup_title")
    padding: 0
    closePolicy: Popup.CloseOnEscape
    z: 2000

    width: Math.min(840, (Overlay.overlay ? Overlay.overlay.width : 960) - 32)
    height: Math.min(660, (Overlay.overlay ? Overlay.overlay.height : 640) - 24)
    x: Math.max(12, (Overlay.overlay.width - width) / 2)
    y: Math.max(12, (Overlay.overlay.height - height) / 2)

    property var rules: []
    property string selectedId: ""
    property var scheduler: ({ enabledCount: 0, scheduledCount: 0, error: "" })
    property bool credentialReady: false
    readonly property bool commonMode: Config.passwordMode === "common"
    readonly property bool passwordReady: dialog.commonMode ? Config.commonPasswordSet
                                                            : dialog.rule.passwordSet

    readonly property bool hasRule: selectedRule !== null
    readonly property var selectedRule: {
        for (let i = 0; i < rules.length; ++i) {
            if (rules[i].id === selectedId)
                return rules[i];
        }
        return null;
    }
    // Safe fallback for bindings that are evaluated while the form is hidden.
    readonly property var rule: selectedRule !== null ? selectedRule : ({
        id: "", name: "", enabled: false, sources: [], excludes: [], passwordSet: false,
        oneFileSystem: false, excludeLarger: "",
        schedule: { enabled: false, mode: "interval", minutes: 60, time: "03:00", weekdays: [] }
    })

    signal passwordRequested(string ruleId, string mode)

    function refresh() {
        rules = Config.rules();
        let exists = false;
        for (let i = 0; i < rules.length; ++i) {
            if (rules[i].id === selectedId)
                exists = true;
        }
        if (!exists) {
            if (rules.length === 0) {
                // "New rule" from the main window can race with the dialog
                // opening; make sure there is always something to edit.
                selectedId = Config.addRule(qsTr("rule_new_name"));
                rules = Config.rules();
            } else {
                selectedId = rules[0].id;
            }
        }
        // Do not read dialog.rule here: its binding may still point at the
        // previous selection until the assignments above settle.
        let selected = null;
        for (let i = 0; i < rules.length; ++i) {
            if (rules[i].id === selectedId)
                selected = rules[i];
        }
        credentialReady = selected !== null && Store.ruleCanRunUnattended(selected.id);
        scheduler = Scheduler.poll();
    }

    function pretty(path) {
        return String(path).replace(App.homePath(), "~");
    }

    function urlToPath(url) {
        return decodeURIComponent(url.toString().replace(/^file:\/\//, ""));
    }

    function dayName(day) {
        const names = [qsTr("weekday_short_mon"), qsTr("weekday_short_tue"),
                       qsTr("weekday_short_wed"), qsTr("weekday_short_thu"),
                       qsTr("weekday_short_fri"), qsTr("weekday_short_sat"),
                       qsTr("weekday_short_sun")];
        return names[day - 1];
    }

    function openRule(id) {
        selectedId = id;
        open();
    }

    // Every change is saved immediately; Save only makes sure edits that are
    // still sitting in a text field are committed before the card closes.
    function save() {
        if (dialog.hasRule) {
            if (nameField.text !== dialog.rule.name)
                Config.setRuleName(dialog.rule.id, nameField.text);
            if (timeField.visible && timeField.text !== dialog.rule.schedule.time)
                Config.setRuleScheduleTime(dialog.rule.id, timeField.text);
            if (patternField.text.trim() !== "") {
                Config.addRuleExclude(dialog.rule.id, patternField.text);
                patternField.text = "";
            }
        }
        dialog.close();
    }

    onOpened: refresh()

    Connections {
        target: Config
        function onRules_changed() {
            if (dialog.opened)
                dialog.refresh();
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

    contentItem: ColumnLayout {
        spacing: 0

        // ---------------------------------------------------------- header ----
        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 20
            Layout.rightMargin: 20
            Layout.topMargin: 16
            spacing: 3

            Label {
                text: qsTr("rules_title")
                font.pixelSize: 19
                font.weight: Font.DemiBold
                color: Kirigami.Theme.textColor
            }
            Label {
                Layout.fillWidth: true
                text: qsTr("rules_hint")
                font.pixelSize: 12
                wrapMode: Text.WordWrap
                opacity: 0.7
                color: Kirigami.Theme.textColor
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.topMargin: 14
            Layout.preferredHeight: 1
            color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                           Kirigami.Theme.textColor.b, 0.15)
        }

        // A plain Flickable instead of ScrollView: the content is a layout
        // whose height is known explicitly, so the whole page (including the
        // list rows) scrolls as one and there is no nested wheel trap.
        Flickable {
                id: formScroll
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                contentWidth: width
                contentHeight: formGrid.implicitHeight + 36
                flickableDirection: Flickable.VerticalFlick
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar {
                    policy: formScroll.contentHeight > formScroll.height
                            ? ScrollBar.AlwaysOn : ScrollBar.AsNeeded
                }

                // Two columns keep "What to copy" and "Excluded paths" next to
                // each other; narrow windows fall back to a single column.
                GridLayout {
                    id: formGrid
                    width: formScroll.width - 56
                    x: 28
                    y: 18
                    columns: dialog.width >= 800 ? 2 : 1
                    columnSpacing: 28
                    rowSpacing: 16
                    enabled: dialog.hasRule

                    ColumnLayout {
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignTop | Qt.AlignLeft
                        spacing: 12

                    // Name + rule-level switch.
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 10

                        TextField {
                            id: nameField
                            Layout.fillWidth: true
                            Layout.minimumWidth: 0
                            font.pixelSize: 15
                            placeholderText: qsTr("rule_name_placeholder")
                            text: dialog.rule.name
                            onEditingFinished: {
                                if (dialog.hasRule && text !== dialog.rule.name) {
                                    Config.setRuleName(dialog.rule.id, text);
                                    dialog.refresh();
                                }
                            }
                        }

                        Label {
                            text: qsTr("rule_enabled")
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }
                        Switch {
                            checked: dialog.rule.enabled
                            onToggled: {
                                if (dialog.hasRule) {
                                    Config.setRuleEnabled(dialog.rule.id, checked);
                                    dialog.refresh();
                                }
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 1
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.15)
                    }

                    // --------------------------------------------- password ----
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 1

                            Label {
                                text: qsTr("rule_password_title")
                                font.pixelSize: 15
                                font.weight: Font.DemiBold
                                color: Kirigami.Theme.textColor
                            }
                            Label {
                                Layout.fillWidth: true
                                text: dialog.commonMode
                                      ? (dialog.passwordReady ? qsTr("rule_password_common_set")
                                                              : qsTr("rule_password_common_missing"))
                                      : (dialog.passwordReady ? qsTr("rule_password_set")
                                                              : qsTr("rule_password_missing"))
                                font.pixelSize: 11
                                wrapMode: Text.WordWrap
                                opacity: 0.7
                                color: dialog.passwordReady ? Kirigami.Theme.textColor
                                                            : Kirigami.Theme.negativeTextColor
                            }
                        }
                        Button {
                            visible: !dialog.commonMode
                            text: dialog.rule.passwordSet ? qsTr("action_change_password")
                                                          : qsTr("action_set_password")
                            icon.name: "dialog-password"
                            enabled: dialog.hasRule && !dialog.rule.synthetic
                            onClicked: dialog.passwordRequested(dialog.rule.id,
                                    dialog.rule.passwordSet ? "change" : "create")
                        }
                        Button {
                            visible: dialog.commonMode
                            text: dialog.passwordReady ? qsTr("action_change_password")
                                                       : qsTr("action_set_password")
                            icon.name: "dialog-password"
                            enabled: dialog.hasRule && !dialog.rule.synthetic
                            onClicked: dialog.passwordRequested(dialog.rule.id, "common")
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        text: dialog.commonMode ? qsTr("rule_password_common_hint")
                                                : qsTr("rule_password_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        visible: dialog.rule.passwordSet && dialog.credentialReady
                        text: qsTr("rule_password_unattended_ready")
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        opacity: 0.6
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        visible: dialog.rule.passwordSet && !dialog.credentialReady
                        text: qsTr("rule_password_unattended_missing")
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        color: Kirigami.Theme.negativeTextColor
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 1
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.15)
                    }

                    // -------------------------------------------- schedule ----
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 6

                        Label {
                            Layout.fillWidth: true
                            text: qsTr("rule_schedule_title")
                            font.pixelSize: 15
                            font.weight: Font.DemiBold
                            elide: Text.ElideRight
                            color: Kirigami.Theme.textColor
                        }
                        Label {
                            text: qsTr("rule_automatic")
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }
                        Switch {
                            checked: dialog.rule.schedule.enabled
                            enabled: dialog.hasRule
                                     && (dialog.rule.schedule.enabled || dialog.credentialReady)
                            onToggled: {
                                if (dialog.hasRule) {
                                    Config.setRuleScheduleEnabled(dialog.rule.id, checked);
                                    dialog.refresh();
                                }
                            }
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        visible: !dialog.credentialReady
                        text: qsTr("rule_schedule_needs_password")
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        color: Kirigami.Theme.negativeTextColor
                    }
                    Label {
                        Layout.fillWidth: true
                        text: qsTr("rule_schedule_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }

                    Flow {
                        Layout.fillWidth: true
                        spacing: 10
                        enabled: dialog.rule.schedule.enabled

                        ComboBox {
                            width: 190
                            font.pixelSize: 13
                            model: [
                                { value: "interval", text: qsTr("rule_mode_interval") },
                                { value: "daily", text: qsTr("rule_mode_daily") },
                                { value: "weekly", text: qsTr("rule_mode_weekly") }
                            ]
                            textRole: "text"
                            valueRole: "value"
                            currentIndex: {
                                for (let i = 0; i < model.length; ++i) {
                                    if (model[i].value === dialog.rule.schedule.mode)
                                        return i;
                                }
                                return 0;
                            }
                            onActivated: (index) => {
                                if (dialog.hasRule) {
                                    Config.setRuleScheduleMode(dialog.rule.id, model[index].value);
                                    dialog.refresh();
                                }
                            }
                        }

                        SpinBox {
                            visible: dialog.rule.schedule.mode === "interval"
                            width: 120
                            from: 5
                            to: 10080
                            stepSize: 5
                            value: dialog.rule.schedule.minutes
                            textFromValue: (value) => qsTr("rule_minutes_value").arg(value)
                            valueFromText: (text) => parseInt(text, 10) || 5
                            onValueModified: {
                                if (dialog.hasRule) {
                                    Config.setRuleScheduleMinutes(dialog.rule.id, value);
                                    dialog.refresh();
                                }
                            }
                        }

                        TextField {
                            id: timeField
                            visible: dialog.rule.schedule.mode !== "interval"
                            width: 92
                            font.pixelSize: 13
                            horizontalAlignment: Text.AlignHCenter
                            placeholderText: qsTr("rule_time_placeholder")
                            text: dialog.rule.schedule.time
                            validator: RegularExpressionValidator {
                                regularExpression: /^([01]?\d|2[0-3]):[0-5]\d$/
                            }
                            onEditingFinished: {
                                if (dialog.hasRule)
                                    Config.setRuleScheduleTime(dialog.rule.id, text);
                                dialog.refresh();
                            }
                        }

                    }
                    Label {
                        Layout.fillWidth: true
                        visible: dialog.rule.schedule.enabled
                                 && dialog.rule.schedule.mode !== "interval"
                        text: qsTr("rule_time_hint")
                        font.pixelSize: 11
                        opacity: 0.6
                        color: Kirigami.Theme.textColor
                    }

                    Flow {
                        Layout.fillWidth: true
                        spacing: 4
                        visible: dialog.rule.schedule.mode === "weekly"
                                 && dialog.rule.schedule.enabled

                        Repeater {
                            model: 7

                            delegate: CheckBox {
                                readonly property int isoDay: index + 1
                                text: dialog.dayName(isoDay)
                                font.pixelSize: 11
                                leftPadding: 2
                                rightPadding: 2
                                spacing: 2
                                checked: dialog.rule.schedule.weekdays.indexOf(isoDay) !== -1
                                enabled: !checked || dialog.rule.schedule.weekdays.length > 1
                                onToggled: {
                                    if (dialog.hasRule) {
                                        Config.setRuleWeekday(dialog.rule.id, isoDay, checked);
                                        dialog.refresh();
                                    }
                                }
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 1
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.15)
                    }

                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignTop | Qt.AlignLeft
                        spacing: 12

                    // --------------------------------------------- sources ----
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 6

                        Label {
                            Layout.fillWidth: true
                            text: qsTr("backup_include_title")
                            font.pixelSize: 15
                            font.weight: Font.DemiBold
                            elide: Text.ElideRight
                            color: Kirigami.Theme.textColor
                        }
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
                    }
                    Label {
                        Layout.fillWidth: true
                        text: qsTr("backup_include_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 0

                        Repeater {
                            model: dialog.rule.sources

                            delegate: Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: 30
                                radius: 4
                                color: sourceHover.containsMouse
                                       ? Qt.rgba(Kirigami.Theme.textColor.r,
                                                 Kirigami.Theme.textColor.g,
                                                 Kirigami.Theme.textColor.b, 0.06)
                                       : "transparent"

                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: 8
                                    anchors.rightMargin: 16
                                    spacing: 8

                                    Kirigami.Icon {
                                        source: "folder"
                                        implicitWidth: 16
                                        implicitHeight: 16
                                    }
                                    Label {
                                        Layout.fillWidth: true
                                        text: dialog.pretty(modelData)
                                        font.pixelSize: 13
                                        elide: Text.ElideMiddle
                                        color: Kirigami.Theme.textColor
                                    }
                                    ToolButton {
                                        implicitWidth: 26
                                        implicitHeight: 26
                                        padding: 0
                                        icon.name: "list-remove"
                                        icon.width: 14
                                        icon.height: 14
                                        onClicked: {
                                            Config.removeRuleSource(dialog.rule.id, index);
                                            dialog.refresh();
                                        }
                                        ToolTip.text: qsTr("rule_remove_source")
                                        ToolTip.visible: hovered
                                    }
                                }

                                MouseArea {
                                    id: sourceHover
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    acceptedButtons: Qt.NoButton
                                }
                            }
                        }
                    }

                    Label {
                        Layout.fillWidth: true
                        visible: dialog.rule.sources.length === 0
                        text: qsTr("backup_no_sources")
                        wrapMode: Text.WordWrap
                        font.pixelSize: 12
                        color: Kirigami.Theme.negativeTextColor
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 1
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.15)
                    }

                    // ------------------------------------------ exclusions ----
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 6

                        Label {
                            Layout.fillWidth: true
                            text: qsTr("backup_exclude_title")
                            font.pixelSize: 15
                            font.weight: Font.DemiBold
                            elide: Text.ElideRight
                            color: Kirigami.Theme.textColor
                        }
                        Button {
                            text: qsTr("backup_add_recommended")
                            icon.name: "list-add"
                            enabled: dialog.hasRule
                            onClicked: {
                                Config.addRecommendedExcludes(dialog.rule.id);
                                dialog.refresh();
                            }
                            ToolTip.text: qsTr("backup_recommended_hint")
                            ToolTip.visible: hovered
                        }
                        Button {
                            text: qsTr("backup_add_exclude")
                            icon.name: "folder-new"
                            onClicked: excludeFolderDialog.open()
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        text: qsTr("backup_exclude_hint")
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        opacity: 0.7
                        color: Kirigami.Theme.textColor
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 6

                        TextField {
                            id: patternField
                            Layout.fillWidth: true
                            font.pixelSize: 13
                            placeholderText: qsTr("backup_pattern_placeholder")
                            onAccepted: {
                                if (text.trim() !== "" && dialog.hasRule) {
                                    Config.addRuleExclude(dialog.rule.id, text);
                                    text = "";
                                    dialog.refresh();
                                }
                            }
                        }
                        Button {
                            text: qsTr("backup_add_pattern")
                            icon.name: "list-add"
                            enabled: patternField.text.trim() !== "" && dialog.hasRule
                            onClicked: {
                                Config.addRuleExclude(dialog.rule.id, patternField.text);
                                patternField.text = "";
                                dialog.refresh();
                            }
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 0

                        Repeater {
                            model: dialog.rule.excludes

                            delegate: Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: 30
                                radius: 4
                                color: excludeHover.containsMouse
                                       ? Qt.rgba(Kirigami.Theme.textColor.r,
                                                 Kirigami.Theme.textColor.g,
                                                 Kirigami.Theme.textColor.b, 0.06)
                                       : "transparent"

                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: 8
                                    anchors.rightMargin: 16
                                    spacing: 8

                                    Kirigami.Icon {
                                        source: String(modelData).indexOf("/") === -1
                                                ? "view-filter" : "folder"
                                        implicitWidth: 16
                                        implicitHeight: 16
                                    }
                                    Label {
                                        Layout.fillWidth: true
                                        text: dialog.pretty(modelData)
                                        font.pixelSize: 13
                                        elide: Text.ElideMiddle
                                        color: Kirigami.Theme.textColor
                                    }
                                    ToolButton {
                                        implicitWidth: 26
                                        implicitHeight: 26
                                        padding: 0
                                        icon.name: "list-remove"
                                        icon.width: 14
                                        icon.height: 14
                                        onClicked: {
                                            Config.removeRuleExclude(dialog.rule.id, index);
                                            dialog.refresh();
                                        }
                                        ToolTip.text: qsTr("rule_remove_exclude")
                                        ToolTip.visible: hovered
                                    }
                                }

                                MouseArea {
                                    id: excludeHover
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    acceptedButtons: Qt.NoButton
                                }
                            }
                        }
                    }

                    Label {
                        Layout.fillWidth: true
                        visible: dialog.rule.excludes.length === 0
                        text: qsTr("rule_no_excludes")
                        wrapMode: Text.WordWrap
                        font.pixelSize: 12
                        opacity: 0.6
                        color: Kirigami.Theme.textColor
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 1
                        color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.15)
                    }

                    Label {
                        Layout.fillWidth: true
                        text: qsTr("rule_advanced")
                        font.pixelSize: 15
                        font.weight: Font.DemiBold
                        color: Kirigami.Theme.textColor
                    }
                    CheckBox {
                        Layout.fillWidth: true
                        text: qsTr("rule_one_filesystem")
                        checked: dialog.rule.oneFileSystem
                        enabled: dialog.hasRule
                        onToggled: {
                            if (dialog.hasRule) {
                                Config.setRuleOneFileSystem(dialog.rule.id, checked);
                                dialog.refresh();
                            }
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        text: qsTr("rule_one_filesystem_hint")
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        opacity: 0.6
                        color: Kirigami.Theme.textColor
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        Label {
                            text: qsTr("rule_exclude_larger")
                            font.pixelSize: 13
                            color: Kirigami.Theme.textColor
                        }
                        TextField {
                            Layout.preferredWidth: 100
                            font.pixelSize: 13
                            placeholderText: qsTr("rule_exclude_larger_placeholder")
                            text: dialog.rule.excludeLarger
                            enabled: dialog.hasRule
                            onEditingFinished: {
                                if (dialog.hasRule && text !== dialog.rule.excludeLarger) {
                                    Config.setRuleExcludeLarger(dialog.rule.id, text);
                                    dialog.refresh();
                                }
                            }
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        text: qsTr("rule_exclude_larger_hint")
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        opacity: 0.6
                        color: Kirigami.Theme.textColor
                    }

                    }
                }
            }
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                           Kirigami.Theme.textColor.b, 0.15)
        }

        // ---------------------------------------------------------- footer ----
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 20
            Layout.rightMargin: 20
            Layout.topMargin: 10
            Layout.bottomMargin: 12
            spacing: 10

            Label {
                Layout.fillWidth: true
                visible: dialog.scheduler.scheduledCount > 0
                text: qsTr("rules_scheduled").arg(dialog.scheduler.scheduledCount)
                font.pixelSize: 11
                elide: Text.ElideRight
                opacity: 0.65
                color: Kirigami.Theme.textColor
            }
            Label {
                Layout.fillWidth: true
                visible: dialog.scheduler.error !== ""
                text: dialog.scheduler.error
                font.pixelSize: 11
                elide: Text.ElideMiddle
                color: Kirigami.Theme.negativeTextColor
            }

            Button {
                text: qsTr("action_cancel")
                onClicked: dialog.close()
            }
            Button {
                text: qsTr("action_save")
                icon.name: "document-save"
                enabled: dialog.hasRule
                onClicked: dialog.save()
            }
        }
    }

    FolderDialog {
        id: includeFolderDialog
        title: qsTr("backup_add_folder")
        onAccepted: {
            if (dialog.hasRule) {
                Config.addRuleSource(dialog.rule.id, dialog.urlToPath(selectedFolder));
                dialog.refresh();
            }
        }
    }
    FileDialog {
        id: includeFileDialog
        title: qsTr("backup_add_file")
        fileMode: FileDialog.OpenFile
        onAccepted: {
            if (dialog.hasRule) {
                Config.addRuleSource(dialog.rule.id, dialog.urlToPath(selectedFile));
                dialog.refresh();
            }
        }
    }
    FolderDialog {
        id: excludeFolderDialog
        title: qsTr("backup_add_exclude")
        onAccepted: {
            if (dialog.hasRule) {
                Config.addRuleExclude(dialog.rule.id, dialog.urlToPath(selectedFolder));
                dialog.refresh();
            }
        }
    }
}
