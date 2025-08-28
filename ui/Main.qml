import QtQuick
import QtQuick.Controls
import QtQuick.Window
import QtQuick.Controls.Material
import QtQuick.Controls.Universal
import WayCast

ApplicationWindow {
  id: win
  visible: false
  width: 600
  height: 400
  flags: Qt.FramelessWindowHint
  property int timeoutInterval: 5000
  
  Shortcut {
    sequence: "Escape"
    onActivated: Qt.quit()
  }
  
  Component.onCompleted: {
    forceActiveFocus()
  }

  Rectangle {
    anchors.fill: parent
    radius: 8
    border.width: 1
    border.color: palette.mid
    color: palette.window
    
    Column {
      anchors.fill: parent
      anchors.margins: 10
      spacing: 5
      
      TextField {
        id: searchField
        width: parent.width
        placeholderText: "Type to search applications..."
        selectByMouse: true
        focus: true
        
        Keys.onDownPressed: listView.incrementCurrentIndex()
        Keys.onUpPressed: listView.decrementCurrentIndex()
        Keys.onReturnPressed: {
          if (listView.currentItem) {
            console.log("Selected:", appModel.data(appModel.index(listView.currentIndex, 0), Qt.UserRole + 2))
          }
        }
      }
      
      ScrollView {
        width: parent.width
        height: parent.height - searchField.height - parent.spacing
        clip: true
        
        ListView {
          id: listView
          model: appModel
          currentIndex: 0
          highlightFollowsCurrentItem: true
          
          highlight: Rectangle {
            color: palette.highlight
            radius: 4
          }
          
          delegate: ItemDelegate {
            width: listView.width
            height: 40
            
            Rectangle {
              anchors.fill: parent
              color: parent.hovered ? palette.alternateBase : "transparent"
              radius: 4
            }
            
            Row {
              anchors.left: parent.left
              anchors.verticalCenter: parent.verticalCenter
              anchors.margins: 10
              spacing: 10
              
              Rectangle {
                width: 24
                height: 24
                color: palette.button
                radius: 4
                
                Text {
                  anchors.centerIn: parent
                  text: "📱"
                  font.pixelSize: 16
                }
              }
              
              Text {
                anchors.verticalCenter: parent.verticalCenter
                text: model.name
                color: palette.text
                font.pixelSize: 14
              }
            }
            
            onClicked: {
              listView.currentIndex = index
              console.log("Clicked:", model.exec)
            }
          }
        }
      }
    }
  }
  
  AppListModel {
    id: appModel
  }
}
