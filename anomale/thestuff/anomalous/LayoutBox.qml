import QtQuick 2.0

ComboBox {
    id: combo
    model: keyboard.layouts
    index: keyboard.currentLayout

    onValueChanged: keyboard.currentLayout = id

    Connections {
        target: keyboard
        onCurrentLayoutChanged: combo.index = keyboard.currentLayout
    }
}
