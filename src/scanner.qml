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
                    scanner.plot.connect(onPlot)
                }
                function onRtlProduct(product) {text = product}
                function onStatus(status) {text = status}
                function onPlot(data) { canvas.rtlData = data; canvas.requestPaint() }
            }
        }
    }

    ColumnLayout {
        id: mainLayout
        anchors.fill: parent
        anchors.margins: margin

        Item {
            id: graphBox
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.minimumHeight: 200
            Layout.minimumWidth: 300

            Canvas {
                id: canvas
                anchors.fill: parent
                property var rtlData

                onWidthChanged: {scanner.resize(width, height)}
                onHeightChanged: {scanner.resize(width, height)}

                onPaint: {
                    if(this.rtlData == null)
                        return
                    var ctx = canvas.getContext('2d');
                    ctx.reset()

                    ctx.lineWidth = 1
                    ctx.fillStyle = Qt.rgba(0, 0, 1, 0.1)
                    ctx.strokeStyle = Qt.rgba(0, 0, 0, 0.3)
                    ctx.beginPath()

                    ctx.moveTo(0, 0)
                    for(var i=0; i<rtlData.length; i++) {
                        ctx.lineTo(i, rtlData[i])
                    }
                    ctx.lineTo(rtlData.length-1, 0) // to make fill area horizontal

                    ctx.closePath()
                    ctx.fill()
                    ctx.stroke()
                }
            }
        }

        GroupBox {
            id: controlsPanel
            anchors.bottom: parent.bottom
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
                    model: [ ]
                    Component.onCompleted: {
                        scanner.gains.connect(onGains)
                    }
                    function onGains(gains) {model = gains}
                }
            }
        }

    }
}