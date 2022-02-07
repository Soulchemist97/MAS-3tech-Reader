from MAS_Reader import *
from GUI.form import Ui_MainWindow

import sys
from PyQt5 import QtWidgets

app = QtWidgets.QApplication(sys.argv)

window = QtWidgets.QMainWindow()
window.setWindowTitle("MAS Reader")

ui_window = Ui_MainWindow()
ui_window.setupUi(window)


def StartClicked():
    # if ui_window.ExcelBox.isOpen:
        # pass
    pass

# ui_window.StartButton.clicked(StartClicked())


if __name__ =="__main__":
 
    window.show()

    sys.exit(app.exec())