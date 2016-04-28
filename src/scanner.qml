import QtQuick 2.2
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.0

ApplicationWindow {
    visible: true
    title: "RTL scanner"
    property int margin: 11
    width: mainLayout.implicitWidth + 2 * margin
    height: mainLayout.implicitHeight + 2 * margin
    minimumWidth: mainLayout.Layout.minimumWidth + 2 * margin
    minimumHeight: mainLayout.Layout.minimumHeight + 2 * margin

    ColumnLayout {
        id: mainLayout
        anchors.fill: parent
        anchors.margins: margin

        GroupBox {
            id: graphBox
            title: "Graph"
            Layout.fillWidth: true

            Canvas {
                id: canvas
                width: 300
                height: 200

                onPaint: {
                    var ctx = canvas.getContext('2d')
                }
            }
        }

        GroupBox {
            RowLayout {
                Button {
                    id: btStartStop
                    text: "Start"
                }

                TextField {
                    id: txtStart
                    placeholderText: "Start Mhz"
                }

                TextField {
                    id: txtStop
                    placeholderText: "Stop Mhz"
                }

                ComboBox {
                    model: [ "gain 1", "gain 2", "gain 3" ]
                }
            }
        }

    }
}
