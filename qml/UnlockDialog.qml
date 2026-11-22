// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

// Password dialog for one backup rule.
//
//  - unlock: open a locked backup with its own password
//  - create: give the rule its first password (and initialize the repository
//            when it does not exist yet)
//  - change: replace the rule's password from the rule editor
//
// A rule without a password needs an authorization password only when the
// repository already exists and no other backup is unlocked.
Dialog {
    id: dialog

    parent: Overlay.overlay
    modal: true
    focus: true
    padding: 18
    closePolicy: dialog.busy ? Popup.NoAutoClose : Popup.CloseOnEscape
    // Above the KustosMenu popups (z: 1000): a password prompt must never be
    // covered by an older context menu.
    z: 2000

    width: Math.min(560, (Overlay.overlay ? Overlay.overlay.width : 800) - 40)
    x: Math.max(12, (Overlay.overlay.width - width) / 2)
    y: Math.max(12, (Overlay.overlay.height - height) / 2)

    property string mode: "unlock"
    property string ruleId: ""
    property string ruleName: ""
    property bool common: false
    property bool authRequired: false
    property bool busy: false
    property bool showPassword: false
    property bool succeeded: false
    property string error: ""

    signal finished(string ruleId, string mode)

    readonly property bool creating: mode === "create" || mode === "change"

    title: common
           ? (mode === "create" ? qsTr("password_common_create_title")
              : mode === "change" ? qsTr("password_common_change_title")
                                  : qsTr("password_common_unlock_title"))
           : (mode === "create" ? qsTr("password_create_title")
              : mode === "change" ? qsTr("password_change_title")
                                  : qsTr("password_unlock_title"))

    function openUnlock(id, name, isCommon) {
        mode = "unlock";
        ruleId = id;
        ruleName = name;
        common = isCommon === true;
        authRequired = false;
        reset();
    }

    function openCreate(id, name, needsAuth, isCommon) {
        mode = "create";
        ruleId = id;
        ruleName = name;
        common = isCommon === true;
        authRequired = needsAuth;
        reset();
    }

    function openChange(id, name, needsAuth) {
        mode = "change";
        ruleId = id;
        ruleName = name;
        common = false;
        authRequired = needsAuth;
        reset();
    }

    // One password for every rule. `changing` picks the change hint; the
    // current password is never needed because a saved credential can
    // authorize a fresh key.
    function openCommon(id, name, changing) {
        mode = changing ? "change" : "create";
        ruleId = id;
        ruleName = name;
        common = true;
        authRequired = Store.status().hasRepo && !Store.anyUnlocked()
                       && !Store.canAuthorizeWithCredential();
        reset();
    }

    function reset() {
        passwordField.text = "";
        authField.text = "";
        confirmField.text = "";
        showPassword = false;
        busy = false;
        succeeded = false;
        error = "";
        open();
        if (authRequired)
            authField.forceActiveFocus();
        else
            passwordField.forceActiveFocus();
    }

    function canSubmit() {
        if (busy || passwordField.text.length === 0)
            return false;
        if (authRequired && authField.text.length === 0)
            return false;
        if (creating && passwordField.text !== confirmField.text)
            return false;
        return true;
    }

    function submit() {
        if (!canSubmit())
            return;
        error = "";
        busy = true;
        if (mode === "unlock")
            Store.unlockRule(ruleId, passwordField.text);
        else if (common)
            Store.setCommonPassword(authRequired ? authField.text : "", passwordField.text);
        else
            Store.setRulePassword(ruleId, authRequired ? authField.text : "", passwordField.text);
    }

    function poll() {
        if (!busy || Store.passwordPending())
            return;
        busy = false;
        if (Store.passwordSucceeded()) {
            if (Store.passwordBindsAll()) {
                if (Store.passwordKeyId() !== "")
                    Config.bindCommonKey(Store.passwordKeyId());
            } else if (Store.passwordNeedsBind() && Store.passwordKeyId() !== "") {
                Config.bindRuleKey(Store.passwordRuleId(), Store.passwordKeyId());
            }
            const id = ruleId;
            const kind = mode;
            succeeded = true;
            close();
            dialog.finished(id, kind);
        } else {
            dialog.error = Store.passwordError();
            if (dialog.error === "")
                dialog.error = qsTr("password_wrong");
        }
    }

    Timer {
        interval: 100
        repeat: true
        running: dialog.busy
        onTriggered: dialog.poll()
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
        spacing: 14

        Label {
            Layout.fillWidth: true
            text: dialog.common
                  ? qsTr("password_for_all_backups")
                  : (dialog.ruleName !== "" ? qsTr("password_for_rule").arg(dialog.ruleName)
                                            : qsTr("password_for_backup"))
            font.pixelSize: 15
            font.weight: Font.DemiBold
            elide: Text.ElideRight
            color: Kirigami.Theme.textColor
        }

        Label {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: dialog.common
                  ? (dialog.mode === "create"
                     ? qsTr("password_common_create_hint")
                     : dialog.mode === "change"
                       ? qsTr("password_common_change_hint")
                       : qsTr("password_common_unlock_hint"))
                  : (dialog.mode === "create"
                     ? (dialog.authRequired ? qsTr("password_create_auth_hint")
                                            : qsTr("password_create_hint"))
                     : dialog.mode === "change"
                       ? qsTr("password_change_hint")
                       : qsTr("password_unlock_hint"))
            font.pixelSize: 13
            color: Kirigami.Theme.textColor
        }

        TextField {
            id: authField
            Layout.fillWidth: true
            visible: dialog.authRequired
            placeholderText: dialog.mode === "change"
                             ? qsTr("password_current_placeholder")
                             : qsTr("password_existing_placeholder")
            echoMode: TextInput.Password
            font.pixelSize: 14
            enabled: !dialog.busy
            onAccepted: {
                if (dialog.canSubmit())
                    dialog.submit();
            }
        }

        TextField {
            id: passwordField
            Layout.fillWidth: true
            placeholderText: dialog.creating
                             ? qsTr("password_new_placeholder")
                             : qsTr("password_placeholder")
            echoMode: dialog.showPassword ? TextInput.Normal : TextInput.Password
            font.pixelSize: 14
            enabled: !dialog.busy
            onAccepted: {
                if (dialog.canSubmit())
                    dialog.submit();
            }
        }

        TextField {
            id: confirmField
            Layout.fillWidth: true
            visible: dialog.creating
            placeholderText: qsTr("password_confirm_placeholder")
            echoMode: dialog.showPassword ? TextInput.Normal : TextInput.Password
            font.pixelSize: 14
            enabled: !dialog.busy
            onAccepted: {
                if (dialog.canSubmit())
                    dialog.submit();
            }
        }

        CheckBox {
            visible: dialog.creating
            text: qsTr("password_show")
            font.pixelSize: 12
            checked: dialog.showPassword
            onToggled: dialog.showPassword = checked
        }

        Label {
            Layout.fillWidth: true
            visible: dialog.creating && confirmField.text.length > 0
                     && passwordField.text !== confirmField.text
            text: qsTr("password_mismatch")
            wrapMode: Text.WordWrap
            font.pixelSize: 11
            color: Kirigami.Theme.negativeTextColor
        }

        Label {
            Layout.fillWidth: true
            visible: dialog.error !== ""
            text: dialog.error
            wrapMode: Text.WordWrap
            font.pixelSize: 12
            color: Kirigami.Theme.negativeTextColor
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Item { Layout.fillWidth: true }
            BusyIndicator {
                running: dialog.busy
                visible: dialog.busy
                implicitWidth: 24
                implicitHeight: 24
            }
            Button {
                text: qsTr("action_cancel")
                enabled: !dialog.busy
                onClicked: dialog.close()
            }
            Button {
                text: dialog.mode === "create" ? qsTr("action_set_password")
                     : dialog.mode === "change" ? qsTr("action_change_password")
                                                : qsTr("action_unlock")
                icon.name: dialog.mode === "unlock" ? "object-unlocked" : "dialog-password"
                enabled: dialog.canSubmit()
                onClicked: dialog.submit()
            }
        }
    }
}
