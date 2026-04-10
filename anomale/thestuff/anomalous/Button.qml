import QtQuick 2.0

Rectangle {
    id: container
    width: 80; height: 30
    radius: 2

    property string text: "Button"
    property color color: "#11ffffff"
    property color textColor: "white"
    property color activeColor: "#44ffffff"
    property color pressedColor: "#66ffffff"
    property alias font: txt.font
    
    // Compatibility for implicitWidth/Height
    property real implicitWidth: txt.implicitWidth + 24
    property real implicitHeight: txt.implicitHeight + 8

    signal clicked()

    border.color: activeFocus || mouseArea.containsMouse ? activeColor : "transparent"
    border.width: 1

    Behavior on color { ColorAnimation { duration: 150 } }
    Behavior on border.color { ColorAnimation { duration: 150 } }

    color: mouseArea.pressed ? pressedColor : (mouseArea.containsMouse ? activeColor : container.color)

    Text {
        id: txt
        anchors.centerIn: parent
        text: container.text
        color: container.textColor
        font.pixelSize: 12
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        onClicked: {
            container.focus = true
            container.clicked()
        }
    }

    Keys.onPressed: {
        if (event.key === Qt.Key_Space || event.key === Qt.Key_Enter || event.key === Qt.Key_Return) {
            container.clicked()
            event.accepted = true
        }
    }
}
