import QtQuick 2.0

FocusScope {
    id: container
    width: 200; height: 30

    property color color: "white"
    property color borderColor: "gray"
    property color focusColor: "blue"
    property color hoverColor: "lightblue"
    property color textColor: "black"
    
    property alias text: txtMain.text
    property alias font: txtMain.font
    property alias echoMode: txtMain.echoMode
    property alias radius: main.radius

    Rectangle {
        id: main
        anchors.fill: parent
        color: container.color
        border.color: container.borderColor
        border.width: 1
        radius: 2

        Behavior on border.color { ColorAnimation { duration: 150 } }
        Behavior on color { ColorAnimation { duration: 150 } }

        states: [
            State {
                name: "hover"; when: mouseArea.containsMouse && !container.activeFocus
                PropertyChanges { target: main; border.color: container.hoverColor }
            },
            State {
                name: "focus"; when: container.activeFocus
                PropertyChanges { target: main; border.color: container.focusColor; border.width: 1.5 }
            }
        ]
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        cursorShape: Qt.IBeamCursor
        hoverEnabled: true
        onClicked: container.focus = true
    }

    TextInput {
        id: txtMain
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        verticalAlignment: TextInput.AlignVCenter
        color: container.textColor
        clip: true
        focus: true
        selectByMouse: true
        selectionColor: container.focusColor
        selectedTextColor: container.color
        passwordCharacter: "\u25cf"
    }
}
