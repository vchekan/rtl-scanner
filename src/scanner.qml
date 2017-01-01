import QtQuick 2.5;
import QtQuick.Window 2.1;
import QtQuick.Controls 1.4;
import QtQuick.Layouts 1.2;

ApplicationWindow {
    visible: true
    title: "RTL scanner"
    property int margin: 11
    width: mainLayout.implicitWidth + 2 * margin
    height: mainLayout.implicitHeight + 2 * margin
    minimumWidth: mainLayout.Layout.minimumWidth + 2 * margin
    minimumHeight: mainLayout.Layout.minimumHeight + 2 * margin

    Component.onCompleted: {
        scanner.InitHarware()
    }

    statusBar: StatusBar {
        RowLayout {
            anchors.fill: parent
            Label {
                id: statusTxt
                text: "..."
                Component.onCompleted: {
                    scanner.showRtlProduct.connect(onRtlProduct)
                    scanner.status.connect(onStatus)
                }
                function onRtlProduct(product) {text = product}
                function onStatus(status) {text = status}
            }
        }
    }

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
                    var ctx = canvas.getContext('2d');
                    ctx.fillStyle = Qt.rgba(1, 0, 0, 1);
                    ctx.fillRect(0, 0, width, height);
                }
            }
        }

        GroupBox {
            RowLayout {
                Button {
                    id: btStartStop
                    text: "Start"
                    onClicked: scanner.start(102.711*1e6, 101e6)
                }

                TextField {
                    id: txtStart
                    placeholderText: "Start Mhz"
                    text: "102.71172"
                }

                TextField {
                    id: txtStop
                    placeholderText: "Stop Mhz"
                }

                ComboBox {
                    id: cbGains
                    model: [ "gain 1", "gain 2", "gain 3" ]
                    Component.onCompleted: {
                        scanner.gains.connect(onGains)
                    }
                    function onGains(gains) {model = gains}
                }
            }
        }

    }
}