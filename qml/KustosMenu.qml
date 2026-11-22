// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import org.kde.kirigami as Kirigami

// In-app context menu, the same look and the same open/close animation as
// Luma's LumaMenu: an overlay popup with 8 px padding, 8 px radius and a
// 150 ms fade. Declare a single Column with KustosMenuButton rows inside it.
Popup {
    id: menu

    // Every in-app menu is shown in the window overlay, so x/y are window
    // coordinates and the menu always stays inside the window.
    parent: Overlay.overlay
    margins: 10
    padding: 8
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    z: 1000

    property Item anchorItem: null
    property point anchorPoint: Qt.point(0, 0)

    function hostWindow() {
        return contentItem ? contentItem.Window.window : null;
    }

    function positionAtAnchor() {
        const win = menu.hostWindow();
        if (!win || !menu.anchorItem)
            return;
        const point = menu.anchorItem.mapToItem(null, menu.anchorPoint.x, menu.anchorPoint.y);
        const height = menu.contentItem
            ? menu.contentItem.implicitHeight + menu.topPadding + menu.bottomPadding
            : menu.height;
        const width = menu.width > 0 ? menu.width : menu.implicitWidth;
        let ny = point.y;
        if (ny + height > win.height - 10)
            ny = Math.max(10, point.y - height);
        menu.x = Math.max(10, Math.min(point.x, Math.max(10, win.width - width - 10)));
        menu.y = Math.max(10, Math.min(ny, Math.max(10, win.height - height - 10)));
    }

    function openAt(item, localX, localY) {
        anchorItem = item;
        anchorPoint = Qt.point(localX, localY);
        open();
        positionAtAnchor();
    }

    onOpened: positionAtAnchor()
    onHeightChanged: positionAtAnchor()

    background: Rectangle {
        // Opaque, like Luma's menus: the browser behind must not show through.
        color: Qt.rgba(Kirigami.Theme.backgroundColor.r,
                       Kirigami.Theme.backgroundColor.g,
                       Kirigami.Theme.backgroundColor.b, 1.0)
        radius: 8
        border.width: 1
        border.color: Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g,
                              Kirigami.Theme.textColor.b, 0.18)
    }

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: 150; easing.type: Easing.InOutQuad }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: 150; easing.type: Easing.InOutQuad }
    }
}
