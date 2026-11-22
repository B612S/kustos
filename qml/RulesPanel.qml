// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Left column of the main window: the backup rules. Selecting a rule filters
// the snapshot list in the middle column; the context menu starts, stops,
// edits, pauses or deletes the rule.
Rectangle {
    id: panel

    color: Kirigami.Theme.backgroundColor

    property var items: []
    property string selectedId: ""
    property string activeRuleId: ""
    property bool backupRunning: false
    property bool busy: false
    property bool menuEnabled: true
    property var contextRule: null
    readonly property int realRuleCount: {
        let count = 0;
        for (let i = 0; i < items.length; ++i) {
            if (!items[i].synthetic)
                count += 1;
        }
        return count;
    }

    signal ruleActivated(string id)
    // Selects the rule without asking for its password; used by the context
    // menu before the user picks an action.
    signal ruleSelected(string id)
    signal runRequested(string id)
    signal stopRequested()
    signal editRequested(string id)
    signal toggleRequested(string id, bool enabled)
    signal deleteRequested(string id, string name)
    signal addRequested()

    function dayName(day) {
        const names = [qsTr("weekday_short_mon"), qsTr("weekday_short_tue"),
                       qsTr("weekday_short_wed"), qsTr("weekday_short_thu"),
                       qsTr("weekday_short_fri"), qsTr("weekday_short_sat"),
                       qsTr("weekday_short_sun")];
        return names[day - 1];
    }

    function weekdayList(days) {
        if (days.length === 7)
            return qsTr("rule_every_day");
        const names = [];
        for (let day = 1; day <= 7; ++day) {
            if (days.indexOf(day) !== -1)
                names.push(dayName(day));
        }
        return names.join(", ");
    }

    function scheduleText(rule) {
        if (rule.synthetic)
            return qsTr("rules_other_hint");
        const schedule = rule.schedule;
        if (!schedule || !schedule.enabled)
            return qsTr("rule_schedule_off");
        if (schedule.mode === "daily")
            return qsTr("rule_daily_at").arg(schedule.time);
        if (schedule.mode === "weekly")
            return qsTr("rule_weekly_at").arg(weekdayList(schedule.weekdays)).arg(schedule.time);
        if (schedule.minutes % 60 === 0)
            return qsTr("schedule_every_hours").arg(schedule.minutes / 60);
        return qsTr("schedule_every_minutes").arg(schedule.minutes);
    }

    function ruleName(rule) {
        return rule.name !== "" ? rule.name : qsTr("rule_untitled");
    }

    function openMenu(row, anchor, x, y) {
        if (!panel.menuEnabled)
            return;
        contextRule = row;
        ruleMenu.openAt(anchor, x, y);
    }

    onMenuEnabledChanged: {
        if (!menuEnabled)
            ruleMenu.close();
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 14
            Layout.rightMargin: 10
            Layout.topMargin: 12
            Layout.bottomMargin: 10
            spacing: 2

            RowLayout {
                Layout.fillWidth: true

                Label {
                    text: qsTr("rules_panel_title")
                    font.pixelSize: 16
                    font.weight: Font.DemiBold
                    color: Kirigami.Theme.textColor
                }
                Item { Layout.fillWidth: true }
                Label {
                    text: qsTr("rules_count").arg(panel.realRuleCount)
                    font.pixelSize: 11
                    opacity: 0.6
                    color: Kirigami.Theme.textColor
                }
                ToolButton {
                    implicitWidth: 28
                    implicitHeight: 28
                    padding: 0
                    icon.name: "list-add"
                    icon.width: 16
                    icon.height: 16
                    onClicked: panel.addRequested()
                    ToolTip.text: qsTr("rule_add")
                    ToolTip.visible: hovered
                }
            }
            Label {
                Layout.fillWidth: true
                text: qsTr("rules_panel_hint")
                font.pixelSize: 11
                wrapMode: Text.WordWrap
                opacity: 0.55
                color: Kirigami.Theme.textColor
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
            model: panel.items

            delegate: Rectangle {
                id: ruleRow

                readonly property bool current: panel.selectedId === modelData.id
                readonly property bool running: panel.backupRunning
                                                    && panel.activeRuleId === modelData.id

                width: list.width
                height: 68
                color: current
                       ? Qt.rgba(Kirigami.Theme.highlightColor.r,
                                 Kirigami.Theme.highlightColor.g,
                                 Kirigami.Theme.highlightColor.b, 0.28)
                       : (rowMouse.containsMouse
                          ? Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                    Kirigami.Theme.textColor.b, 0.07)
                          : "transparent")

                Rectangle {
                    visible: ruleRow.current
                    anchors.left: parent.left
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    anchors.topMargin: 8
                    anchors.bottomMargin: 8
                    width: 3
                    radius: 1.5
                    color: Kirigami.Theme.highlightColor
                }

                ColumnLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: 18
                    anchors.topMargin: 7
                    anchors.bottomMargin: 7
                    spacing: 1

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 6

                        Kirigami.Icon {
                            visible: modelData.locked && !modelData.synthetic
                            source: "object-locked"
                            implicitWidth: 14
                            implicitHeight: 14
                            opacity: 0.65
                        }
                        Label {
                            Layout.fillWidth: true
                            text: panel.ruleName(modelData)
                            font.pixelSize: 14
                            font.weight: ruleRow.current ? Font.DemiBold : Font.Normal
                            elide: Text.ElideRight
                            color: Kirigami.Theme.textColor
                            opacity: modelData.enabled || modelData.synthetic ? 1.0 : 0.55
                        }
                        BusyIndicator {
                            visible: ruleRow.running
                            running: ruleRow.running
                            implicitWidth: 16
                            implicitHeight: 16
                        }
                        Label {
                            visible: !modelData.enabled && !modelData.synthetic
                            text: qsTr("rule_paused")
                            font.pixelSize: 10
                            opacity: 0.6
                            color: Kirigami.Theme.textColor
                        }
                    }
                    Label {
                        Layout.fillWidth: true
                        text: panel.scheduleText(modelData)
                        font.pixelSize: 11
                        elide: Text.ElideRight
                        opacity: 0.62
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        Layout.fillWidth: true
                        text: modelData.synthetic
                              ? qsTr("rules_other_hint")
                              : modelData.locked
                                ? (modelData.passwordSet ? qsTr("rule_locked")
                                                         : qsTr("rule_no_password"))
                                : qsTr("timeline_count").arg(modelData.snapshotCount)
                                  + (modelData.lastBackupText
                                     ? " · " + qsTr("rule_last_backup").arg(modelData.lastBackupText)
                                     : "")
                        font.pixelSize: 10
                        elide: Text.ElideRight
                        opacity: modelData.staleBackup ? 0.9 : 0.45
                        color: modelData.staleBackup ? Kirigami.Theme.negativeTextColor
                                                     : Kirigami.Theme.textColor
                    }
                }

                MouseArea {
                    id: rowMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: (mouse) => {
                        if (mouse.button === Qt.RightButton) {
                            panel.ruleSelected(modelData.id);
                            if (!modelData.synthetic)
                                panel.openMenu(modelData, ruleRow, mouse.x, mouse.y);
                        } else {
                            panel.ruleActivated(modelData.id);
                        }
                    }
                    onDoubleClicked: {
                        if (!modelData.synthetic)
                            panel.editRequested(modelData.id);
                    }
                }
            }

            ScrollBar.vertical: ScrollBar {}
        }

        Label {
            Layout.fillWidth: true
            Layout.margins: 14
            visible: panel.items.length === 0
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: qsTr("rules_empty_panel_hint")
            font.pixelSize: 11
            opacity: 0.6
            color: Kirigami.Theme.textColor
        }

        Button {
            Layout.fillWidth: true
            Layout.leftMargin: 12
            Layout.rightMargin: 12
            Layout.bottomMargin: 12
            text: qsTr("rule_add")
            icon.name: "list-add"
            onClicked: panel.addRequested()
        }
    }

    KustosMenu {
        id: ruleMenu
        width: 240

        Column {
            width: parent.width

            KustosMenuButton {
                text: qsTr("menu_unlock_rule")
                visible: panel.contextRule !== null && !panel.contextRule.synthetic
                         && panel.contextRule.locked
                isEnabled: visible && !panel.busy
                onClicked: {
                    const id = panel.contextRule.id;
                    ruleMenu.close();
                    panel.ruleActivated(id);
                }
            }
            KustosMenuButton {
                text: qsTr("action_back_up_now")
                visible: panel.contextRule !== null && !panel.contextRule.synthetic
                isEnabled: visible && !panel.busy
                onClicked: {
                    const id = panel.contextRule.id;
                    ruleMenu.close();
                    panel.runRequested(id);
                }
            }
            KustosMenuButton {
                text: qsTr("action_stop_backup")
                visible: panel.contextRule !== null && !panel.contextRule.synthetic
                isEnabled: visible && panel.backupRunning
                           && panel.activeRuleId === panel.contextRule.id
                onClicked: {
                    ruleMenu.close();
                    panel.stopRequested();
                }
            }
            KustosMenuButton {
                text: panel.contextRule !== null && panel.contextRule.enabled
                      ? qsTr("menu_pause_rule") : qsTr("menu_enable_rule")
                visible: panel.contextRule !== null && !panel.contextRule.synthetic
                isEnabled: visible && !panel.busy
                onClicked: {
                    const id = panel.contextRule.id;
                    const enabled = !panel.contextRule.enabled;
                    ruleMenu.close();
                    panel.toggleRequested(id, enabled);
                }
            }
            KustosMenuButton {
                text: qsTr("menu_edit_rule")
                visible: panel.contextRule !== null && !panel.contextRule.synthetic
                isEnabled: visible && !panel.busy
                onClicked: {
                    const id = panel.contextRule.id;
                    ruleMenu.close();
                    panel.editRequested(id);
                }
            }
            Rectangle {
                width: parent.width
                height: 1
                visible: panel.contextRule !== null && !panel.contextRule.synthetic
                color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                               Kirigami.Theme.textColor.b, 0.15)
            }
            KustosMenuButton {
                text: qsTr("menu_delete_rule")
                visible: panel.contextRule !== null && !panel.contextRule.synthetic
                isEnabled: visible && !panel.busy
                destructive: true
                onClicked: {
                    const id = panel.contextRule.id;
                    const name = panel.ruleName(panel.contextRule);
                    ruleMenu.close();
                    panel.deleteRequested(id, name);
                }
            }
        }
    }
}
