// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import org.kde.kirigami as Kirigami

// Row of the in-app context menus (KustosMenu), adapted from Luma's
// LumaMenuButton: a 32 px row highlighted on hover, with an optional arrow,
// check mark and tooltip.
Rectangle {
    id: item

    property string text: ""
    property string tooltipText: ""
    property bool hasArrow: false
    property bool isEnabled: true
    // Destructive action (delete, remove): red label.
    property bool destructive: false
    // Selected entry (current destination, active policy).
    property bool checked: false

    readonly property color selectionBackgroundColor: Qt.rgba(Kirigami.Theme.highlightColor.r,
                                                              Kirigami.Theme.highlightColor.g,
                                                              Kirigami.Theme.highlightColor.b, 0.35)
    readonly property int leftMargin: 12
    readonly property int rightMargin: 12

    signal clicked()

    readonly property color labelColor: {
        if (!item.isEnabled)
            return Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                           Kirigami.Theme.textColor.b, 0.4);
        return item.destructive ? Kirigami.Theme.negativeTextColor : Kirigami.Theme.textColor;
    }

    width: parent ? parent.width : 0
    height: 32
    radius: 4
    color: item.checked
           ? item.selectionBackgroundColor
           : ((hoverArea.containsMouse && item.isEnabled)
              ? Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                        Kirigami.Theme.textColor.b, 0.1)
              : "transparent")

    Rectangle {
        visible: item.checked
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.topMargin: 4
        anchors.bottomMargin: 4
        width: 3
        radius: 1.5
        color: Kirigami.Theme.highlightColor
    }

    Text {
        id: label
        text: item.text
        color: item.labelColor
        textFormat: Text.PlainText
        font.pixelSize: 14
        font.weight: item.checked ? Font.DemiBold : Font.Normal
        elide: Text.ElideRight
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: item.leftMargin
        anchors.right: arrow.visible ? arrow.left : parent.right
        anchors.rightMargin: item.rightMargin
    }

    Text {
        id: arrow
        text: ">"
        visible: item.hasArrow
        color: item.labelColor
        font.pixelSize: 14
        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        anchors.rightMargin: item.rightMargin
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        onClicked: { if (item.isEnabled) item.clicked() }
    }

    ToolTip.visible: hoverArea.containsMouse && item.tooltipText !== ""
    ToolTip.text: item.tooltipText
    ToolTip.delay: 500
}
