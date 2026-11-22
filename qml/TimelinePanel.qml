// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Vertical timeline: day headers and snapshots, newest first.
//
// Panels use backgroundColor together with textColor (never a "alternative"
// color): both come from the same Kirigami color set, so labels stay readable
// in light and dark Plasma themes alike.
Rectangle {
    id: panel

    color: Kirigami.Theme.backgroundColor

    property var items: []
    property string selectedId: ""
    property bool loading: false
    property string heading: ""
    property string emptyText: ""
    property bool showRuleName: false
    property bool locked: false
    property bool hasPassword: true
    property bool menuEnabled: true

    signal snapshotActivated(string id)
    signal deleteRequested(string id)
    signal pinRequested(string id, bool pinned)
    signal unlockRequested()

    onMenuEnabledChanged: {
        if (!menuEnabled)
            snapshotMenu.close();
    }

    function snapshotSubtitle(snapshot) {
        // The repository growth (compressed, deduplicated) is what costs disk
        // space; the raw new-data figure is only shown in the tooltip.
        let text = qsTr("snapshot_on_disk").arg(snapshot.addedPackedText);
        if (panel.showRuleName && snapshot.ruleName !== undefined && snapshot.ruleName !== "")
            text += " · " + snapshot.ruleName;
        return text;
    }

    function snapshotCount() {
        let count = 0;
        for (let i = 0; i < items.length; ++i) {
            if (!items[i].header)
                count += 1;
        }
        return count;
    }

    ColumnLayout {
        id: mainLayout
        anchors.fill: parent
        spacing: 0

        ColumnLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 14
            Layout.rightMargin: 14
            Layout.topMargin: 12
            Layout.bottomMargin: 10
            spacing: 2

            RowLayout {
                Layout.fillWidth: true

                Label {
                    Layout.fillWidth: true
                    text: panel.heading !== ""
                          ? qsTr("timeline_rule_title").arg(panel.heading)
                          : qsTr("timeline_title")
                    font.pixelSize: 16
                    font.weight: Font.DemiBold
                    elide: Text.ElideRight
                    color: Kirigami.Theme.textColor
                }
                Label {
                    visible: !panel.locked
                    text: qsTr("timeline_count").arg(panel.snapshotCount())
                    font.pixelSize: 12
                    opacity: 0.6
                    color: Kirigami.Theme.textColor
                }
            }
            Label {
                Layout.fillWidth: true
                text: qsTr("timeline_hint")
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
            visible: !panel.locked
            model: panel.items

            delegate: Rectangle {
                id: rowItem

                readonly property bool isHeader: modelData.header
                readonly property bool isSelected: !isHeader && panel.selectedId === modelData.snapshot.id

                width: list.width
                height: isHeader ? 30 : 58
                color: isHeader
                       ? "transparent"
                       : (isSelected
                          ? Qt.rgba(Kirigami.Theme.highlightColor.r,
                                    Kirigami.Theme.highlightColor.g,
                                    Kirigami.Theme.highlightColor.b, 0.30)
                          : (rowMouse.containsMouse
                             ? Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                                       Kirigami.Theme.textColor.b, 0.07)
                             : "transparent"))

                Rectangle {
                    visible: rowItem.isSelected
                    anchors.left: parent.left
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    anchors.topMargin: 8
                    anchors.bottomMargin: 8
                    width: 3
                    radius: 1.5
                    color: Kirigami.Theme.highlightColor
                }

                Label {
                    visible: rowItem.isHeader
                    anchors.left: parent.left
                    anchors.leftMargin: 14
                    anchors.verticalCenter: parent.verticalCenter
                    text: rowItem.isHeader ? modelData.label : ""
                    font.pixelSize: 11
                    font.weight: Font.DemiBold
                    opacity: 0.55
                    color: Kirigami.Theme.textColor
                }

                RowLayout {
                    visible: !rowItem.isHeader
                    anchors.fill: parent
                    anchors.leftMargin: 14
                    anchors.rightMargin: 18
                    spacing: 10

                    Label {
                        text: rowItem.isHeader ? "" : modelData.snapshot.timeText
                        font.pixelSize: 16
                        font.weight: Font.DemiBold
                        color: Kirigami.Theme.textColor
                    }
                    Label {
                        visible: !rowItem.isHeader
                        Layout.fillWidth: true
                        text: panel.snapshotSubtitle(modelData.snapshot)
                        font.pixelSize: 12
                        elide: Text.ElideRight
                        opacity: 0.65
                        color: Kirigami.Theme.textColor

                        ToolTip.delay: 600
                        ToolTip.visible: rowMouse.containsMouse && !rowItem.isHeader
                        ToolTip.text: qsTr("snapshot_added_tooltip")
                                      .arg(modelData.snapshot.addedText)
                                      .arg(modelData.snapshot.addedPackedText)
                                      + "\n"
                                      + qsTr("snapshot_changes_tooltip")
                                        .arg(modelData.snapshot.files)
                                        .arg(modelData.snapshot.filesNew)
                                        .arg(modelData.snapshot.filesChanged)
                                        .arg(modelData.snapshot.durationText)
                    }
                    Label {
                        visible: !rowItem.isHeader && modelData.snapshot.pinned
                        text: "★"
                        font.pixelSize: 13
                        opacity: 0.8
                        color: Kirigami.Theme.highlightColor
                    }
                    Rectangle {
                        visible: !rowItem.isHeader && modelData.snapshot.isLatest && panel.width > 250
                        radius: 4
                        color: Qt.rgba(Kirigami.Theme.highlightColor.r,
                                       Kirigami.Theme.highlightColor.g,
                                       Kirigami.Theme.highlightColor.b, 0.25)
                        implicitWidth: latestLabel.implicitWidth + 12
                        implicitHeight: latestLabel.implicitHeight + 4

                        Label {
                            id: latestLabel
                            anchors.centerIn: parent
                            text: qsTr("snapshot_latest")
                            font.pixelSize: 11
                            color: Kirigami.Theme.textColor
                        }
                    }
                }

                MouseArea {
                    id: rowMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: (mouse) => {
                        if (rowItem.isHeader)
                            return;
                        if (mouse.button === Qt.RightButton) {
                            if (!panel.menuEnabled)
                                return;
                            snapshotMenu.snapshotId = modelData.snapshot.id;
                            snapshotMenu.pinned = modelData.snapshot.pinned;
                            snapshotMenu.openAt(rowItem, mouse.x, mouse.y);
                        } else {
                            panel.snapshotActivated(modelData.snapshot.id);
                        }
                    }
                }
            }

            ScrollBar.vertical: ScrollBar {}
        }

        Label {
            Layout.fillWidth: true
            Layout.margins: 16
            visible: panel.snapshotCount() === 0 && !panel.loading && !panel.locked
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: panel.emptyText !== "" ? panel.emptyText : qsTr("timeline_empty")
            font.pixelSize: 12
            opacity: 0.6
            color: Kirigami.Theme.textColor
        }

        // With the snapshot list hidden (nothing unlocked), the layout would
        // centre the header vertically. Keep it at the top instead.
        Item {
            Layout.fillHeight: true
            visible: panel.locked
        }
    }

    ColumnLayout {
        anchors.centerIn: parent
        width: Math.min(280, parent.width - 40)
        spacing: 10
        visible: panel.locked && !panel.loading
        z: 3

        Kirigami.Icon {
            source: "object-locked"
            implicitWidth: 40
            implicitHeight: 40
            Layout.alignment: Qt.AlignHCenter
            opacity: 0.7
        }
        Label {
            Layout.fillWidth: true
            text: qsTr("timeline_locked_title")
            horizontalAlignment: Text.AlignHCenter
            font.pixelSize: 14
            font.weight: Font.DemiBold
            color: Kirigami.Theme.textColor
        }
        Label {
            Layout.fillWidth: true
            text: panel.hasPassword ? qsTr("timeline_locked_hint")
                                    : qsTr("timeline_no_password_hint")
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            font.pixelSize: 12
            opacity: 0.7
            color: Kirigami.Theme.textColor
        }
        Button {
            Layout.alignment: Qt.AlignHCenter
            text: panel.hasPassword ? qsTr("action_unlock") : qsTr("action_set_password")
            icon.name: panel.hasPassword ? "object-unlocked" : "dialog-password"
            onClicked: panel.unlockRequested()
        }
    }

    ColumnLayout {
        anchors.centerIn: parent
        width: Math.min(260, parent.width - 40)
        spacing: 10
        visible: panel.loading
        z: 2

        BusyIndicator {
            running: panel.loading
            Layout.alignment: Qt.AlignHCenter
        }
        Label {
            Layout.fillWidth: true
            text: qsTr("timeline_loading")
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            font.pixelSize: 12
            color: Kirigami.Theme.textColor
        }
    }

    KustosMenu {
        id: snapshotMenu
        width: 230

        property string snapshotId: ""
        property bool pinned: false

        Column {
            width: parent.width
            KustosMenuButton {
                text: snapshotMenu.pinned ? qsTr("menu_unpin_snapshot")
                                          : qsTr("menu_pin_snapshot")
                onClicked: {
                    const id = snapshotMenu.snapshotId;
                    const pinned = !snapshotMenu.pinned;
                    snapshotMenu.close();
                    panel.pinRequested(id, pinned);
                }
            }
            KustosMenuButton {
                text: qsTr("menu_copy_id")
                onClicked: {
                    App.copyText(snapshotMenu.snapshotId);
                    snapshotMenu.close();
                }
            }
            KustosMenuButton {
                text: qsTr("menu_delete_snapshot")
                destructive: true
                onClicked: {
                    snapshotMenu.close();
                    panel.deleteRequested(snapshotMenu.snapshotId);
                }
            }
        }
    }
}
