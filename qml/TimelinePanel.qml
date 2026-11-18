// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Vertical timeline of snapshots, grouped by day (ListView sections), newest
// first. This is the Time Machine "star field" reduced to what a desktop list
// can usefully show: time, date, how much was added and which snapshot is the
// current one.
Rectangle {
    id: panel

    color: Kirigami.Theme.alternativeBackgroundColor

    property string selectedId: ""
    signal snapshotActivated(string id)
    signal deleteRequested(string id)

    function dayLabel(iso) {
        const parts = iso.split("-");
        if (parts.length !== 3)
            return iso;
        const date = new Date(Number(parts[0]), Number(parts[1]) - 1, Number(parts[2]));
        const now = new Date();
        const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
        const diff = Math.round((today.getTime() - date.getTime()) / 86400000);
        if (diff === 0)
            return qsTr("day_today");
        if (diff === 1)
            return qsTr("day_yesterday");
        return Qt.formatDate(date, "dddd, d MMMM yyyy");
    }

    function statusColor(status) {
        if (status === "failed")
            return Kirigami.Theme.negativeTextColor;
        if (status === "warning")
            return "#f5a623";
        return Kirigami.Theme.positiveTextColor;
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 12
            Layout.rightMargin: 12
            Layout.topMargin: 10
            Layout.bottomMargin: 4

            Label {
                text: qsTr("timeline_title")
                font.pixelSize: 16
                font.weight: Font.DemiBold
                color: Kirigami.Theme.textColor
            }
            Item { Layout.fillWidth: true }
            Label {
                text: qsTr("timeline_count").arg(Timeline.count)
                font.pixelSize: 12
                opacity: 0.6
                color: Kirigami.Theme.textColor
            }
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: Timeline
            section.property: "dayKey"
            section.criteria: ViewSection.FullString
            section.delegate: Rectangle {
                width: list.width
                height: 34
                color: "transparent"

                Label {
                    anchors.left: parent.left
                    anchors.leftMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    text: panel.dayLabel(section)
                    font.pixelSize: 12
                    font.weight: Font.DemiBold
                    opacity: 0.55
                    color: Kirigami.Theme.textColor
                }
            }
            delegate: ItemDelegate {
                id: snapshotRow

                width: list.width
                height: 62
                highlighted: panel.selectedId === model.id

                contentItem: RowLayout {
                    spacing: 10

                    Rectangle {
                        Layout.alignment: Qt.AlignVCenter
                        width: 8
                        height: 8
                        radius: 4
                        color: panel.statusColor(model.status)
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 1

                        Label {
                            text: model.timeText
                            font.pixelSize: 15
                            font.weight: Font.DemiBold
                            color: Kirigami.Theme.textColor
                        }
                        Label {
                            text: (model.status === "failed")
                                  ? qsTr("snapshot_status_failed")
                                  : qsTr("snapshot_added").arg(App.formatBytes(model.addedBytes))
                            font.pixelSize: 12
                            opacity: 0.65
                            elide: Text.ElideRight
                            color: Kirigami.Theme.textColor
                        }
                    }

                    Rectangle {
                        visible: model.isLatest
                        Layout.alignment: Qt.AlignVCenter
                        radius: 4
                        color: Qt.rgba(Kirigami.Theme.highlightColor.r,
                                       Kirigami.Theme.highlightColor.g,
                                       Kirigami.Theme.highlightColor.b, 0.18)
                        implicitWidth: latestLabel.implicitWidth + 12
                        implicitHeight: latestLabel.implicitHeight + 4

                        Label {
                            id: latestLabel
                            anchors.centerIn: parent
                            text: qsTr("snapshot_latest")
                            font.pixelSize: 11
                            color: Kirigami.Theme.highlightColor
                        }
                    }
                }

                onClicked: panel.snapshotActivated(model.id)

                TapHandler {
                    acceptedButtons: Qt.RightButton
                    onTapped: {
                        snapshotMenu.snapshotId = model.id;
                        snapshotMenu.openAt(snapshotRow, snapshotRow.width / 2, snapshotRow.height / 2);
                    }
                }
            }

            ScrollBar.vertical: ScrollBar {}
        }

        Label {
            Layout.fillWidth: true
            Layout.margins: 16
            visible: Timeline.count === 0
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: qsTr("timeline_empty")
            font.pixelSize: 12
            opacity: 0.6
            color: Kirigami.Theme.textColor
        }
    }

    AionMenu {
        id: snapshotMenu
        width: 230

        property string snapshotId: ""

        Column {
            width: parent.width
            AionMenuButton {
                text: qsTr("menu_reveal_snapshot")
                onClicked: {
                    App.openPath(Store.snapshotPath(snapshotMenu.snapshotId));
                    snapshotMenu.close();
                }
            }
            AionMenuButton {
                text: qsTr("menu_copy_path")
                onClicked: {
                    App.copyText(Store.snapshotPath(snapshotMenu.snapshotId));
                    snapshotMenu.close();
                }
            }
            AionMenuButton {
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
