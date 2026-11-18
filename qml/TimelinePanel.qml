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

    signal snapshotActivated(string id)
    signal deleteRequested(string id)

    function snapshotCount() {
        let count = 0;
        for (let i = 0; i < items.length; ++i) {
            if (!items[i].header)
                count += 1;
        }
        return count;
    }

    ColumnLayout {
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
                    anchors.rightMargin: 10
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
                        text: qsTr("snapshot_added").arg(modelData.snapshot.addedText)
                        font.pixelSize: 12
                        elide: Text.ElideRight
                        opacity: 0.65
                        color: Kirigami.Theme.textColor
                    }
                    Rectangle {
                        visible: !rowItem.isHeader && modelData.snapshot.isLatest
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
                            snapshotMenu.snapshotId = modelData.snapshot.id;
                            snapshotMenu.openAt(rowItem, rowItem.width / 2, rowItem.height / 2);
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
            visible: panel.snapshotCount() === 0
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: qsTr("timeline_empty")
            font.pixelSize: 12
            opacity: 0.6
            color: Kirigami.Theme.textColor
        }
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
