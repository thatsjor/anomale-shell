import QtQuick 2.0

FocusScope {
    id: container
    width: 200; height: 30

    property color color: "white"
    property color borderColor: "gray"
    property color focusColor: "blue"
    property color hoverColor: "lightblue"
    property color textColor: "black"
    property alias arrowIcon: arrowIcon.source
    
    property var model
    property int index: 0
    property string text: (model && model.length > index) ? (model[index].name || model[index].shortName || "") : ""

    signal valueChanged(int id)

    Rectangle {
        id: main
        anchors.fill: parent
        color: container.color
        border.color: container.borderColor
        border.width: 1
        radius: 2

        Behavior on border.color { ColorAnimation { duration: 150 } }

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

    Text {
        id: mainText
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 30
        verticalAlignment: Text.AlignVCenter
        color: container.textColor
        text: container.text
        elide: Text.ElideRight
        font.pixelSize: 13
    }

    Rectangle {
        id: arrow
        anchors.right: parent.right
        width: 30; height: parent.height
        color: "transparent"

        Image {
            id: arrowIcon
            anchors.centerIn: parent
            width: 12; height: 12
            fillMode: Image.PreserveAspectFit
            smooth: true
        }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        onClicked: {
            container.focus = true
            listViewContainer.visible = !listViewContainer.visible
        }
    }

    Rectangle {
        id: listViewContainer
        width: parent.width
        height: Math.min(listView.contentHeight, 200)
        anchors.top: parent.bottom
        anchors.topMargin: 2
        color: container.color
        border.color: container.borderColor
        border.width: 1
        visible: false
        z: 1000

        ListView {
            id: listView
            anchors.fill: parent
            model: container.model
            clip: true
            delegate: Rectangle {
                width: listView.width; height: 30
                color: model.index === listView.currentIndex ? container.hoverColor : "transparent"
                
                Text {
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    verticalAlignment: Text.AlignVCenter
                    color: container.textColor
                    text: modelData ? (modelData.name || modelData.shortName || "") : (name || "")
                    font.pixelSize: 13
                }

                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true
                    onEntered: listView.currentIndex = index
                    onClicked: {
                        container.index = index
                        container.valueChanged(index)
                        listViewContainer.visible = false
                    }
                }
            }
        }
    }

    onFocusChanged: {
        if (!container.activeFocus) {
            listViewContainer.visible = false
        }
    }
}
