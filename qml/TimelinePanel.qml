// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Vertical timeline: day headers and snapshots, newest first.
Rectangle {
    id: panel

    color: Kirigami.Theme.alternativeBackgroundColor

    property var items: []
    property string selectedId: ""

    signal snapshotActivated(string id)
    signal deleteRequested(string id)

    function statusColor(state) {
        if (state === "failed")
            return Kirigami.Theme.negativeTextColor;
        if (state === "warning")
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
                text: qsTr("timeline_count").arg(panel.snapshotCount())
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
            model: panel.items

            delegate: Rectangle {
                id: rowItem

                width: list.width
                height: modelData.header ? 34 : 62
                color: "transparent"

                Label {
                    visible: modelData.header
                    anchors.left: parent.left
                    anchors.leftMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    text: modelData.header ? modelData.label : ""
                    font.pixelSize: 12
                    font.weight: Font.DemiBold
                    opacity: 0.55
                    color: Kirigami.Theme.textColor
                }

                ItemDelegate {
                    id: snapshotRow
                    visible: !modelData.header
                    anchors.fill: parent
                    highlighted: !modelData.header && panel.selectedId === modelData.snapshot.id

                    contentItem: RowLayout {
                        spacing: 10

                        Rectangle {
                            Layout.alignment: Qt.AlignVCenter
                            width: 8
                            height: 8
                            radius: 4
                            color: modelData.header ? "transparent" : panel.statusColor("ok")
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 1

                            Label {
                                text: modelData.header ? "" : modelData.snapshot.timeText
                                font.pixelSize: 15
                                font.weight: Font.DemiBold
                                color: Kirigami.Theme.textColor
                            }
                            Label {
                                text: modelData.header ? "" : qsTr("snapshot_added").arg(modelData.snapshot.addedText)
                                font.pixelSize: 12
                                opacity: 0.65
                                elide: Text.ElideRight
                                color: Kirigami.Theme.textColor
                            }
                        }

                        Rectangle {
                            visible: !modelData.header && modelData.snapshot.isLatest
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

                    onClicked: {
                        if (!modelData.header)
                            panel.snapshotActivated(modelData.snapshot.id);
                    }

                    TapHandler {
                        acceptedButtons: Qt.RightButton
                        onTapped: {
                            if (modelData.header)
                                return;
                            snapshotMenu.snapshotId = modelData.snapshot.id;
                            snapshotMenu.openAt(snapshotRow, snapshotRow.width / 2, snapshotRow.height / 2);
                        }
                    }
                }
            }

            ScrollBar.vertical: ScrollBar {}
        }

        Label {
            Layout.fillWidth: true
            Layout.margins: 16
            visible: panel.snapshotCount() === 0
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: qsTr("timeline_empty")
            font.pixelSize: 12
            opacity: 0.6
            color: Kirigami.Theme.textColor
        }
    }

    function snapshotCount() {
        let count = 0;
        for (let i = 0; i < items.length; ++i) {
            if (!items[i].header)
                count += 1;
        }
        return count;
    }

    KustosMenu {
        id: snapshotMenu
        width: 230

        property string snapshotId: ""

        Column {
            width: parent.width
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
