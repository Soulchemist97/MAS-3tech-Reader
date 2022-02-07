# -*- coding: utf-8 -*-

################################################################################
## Form generated from reading UI file 'form.ui'
##
## Created by: Qt User Interface Compiler version 6.2.1
##
## WARNING! All changes made in this file will be lost when recompiling UI file!
################################################################################

from PySide6.QtCore import (QCoreApplication, QDate, QDateTime, QLocale,
    QMetaObject, QObject, QPoint, QRect,
    QSize, QTime, QUrl, Qt)
from PySide6.QtGui import (QAction, QBrush, QColor, QConicalGradient,
    QCursor, QFont, QFontDatabase, QGradient,
    QIcon, QImage, QKeySequence, QLinearGradient,
    QPainter, QPalette, QPixmap, QRadialGradient,
    QTransform)
from PySide6.QtWidgets import (QApplication, QCheckBox, QHeaderView, QMainWindow,
    QMenu, QMenuBar, QPushButton, QRadioButton,
    QSizePolicy, QStatusBar, QTextEdit, QTreeWidget,
    QTreeWidgetItem, QWidget)

class Ui_MainWindow(object):
    def setupUi(self, MainWindow):
        if not MainWindow.objectName():
            MainWindow.setObjectName(u"MainWindow")
        MainWindow.resize(849, 533)
        self.actionDatei = QAction(MainWindow)
        self.actionDatei.setObjectName(u"actionDatei")
        self.centralwidget = QWidget(MainWindow)
        self.centralwidget.setObjectName(u"centralwidget")
        self.ExcelBox = QCheckBox(self.centralwidget)
        self.ExcelBox.setObjectName(u"ExcelBox")
        self.ExcelBox.setGeometry(QRect(40, 270, 121, 26))
        self.PDFBox = QCheckBox(self.centralwidget)
        self.PDFBox.setObjectName(u"PDFBox")
        self.PDFBox.setGeometry(QRect(40, 300, 101, 26))
        self.RemoveButton = QRadioButton(self.centralwidget)
        self.RemoveButton.setObjectName(u"RemoveButton")
        self.RemoveButton.setGeometry(QRect(40, 190, 301, 21))
        self.CopyButton = QRadioButton(self.centralwidget)
        self.CopyButton.setObjectName(u"CopyButton")
        self.CopyButton.setGeometry(QRect(40, 220, 112, 26))
        self.IndexButton = QRadioButton(self.centralwidget)
        self.IndexButton.setObjectName(u"IndexButton")
        self.IndexButton.setGeometry(QRect(40, 150, 131, 26))
        self.treeWidget = QTreeWidget(self.centralwidget)
        self.treeWidget.setObjectName(u"treeWidget")
        self.treeWidget.setGeometry(QRect(350, 150, 461, 311))
        self.StartButton = QPushButton(self.centralwidget)
        self.StartButton.setObjectName(u"StartButton")
        self.StartButton.setGeometry(QRect(30, 380, 171, 71))
        self.PathBox = QTextEdit(self.centralwidget)
        self.PathBox.setObjectName(u"PathBox")
        self.PathBox.setGeometry(QRect(40, 80, 771, 41))
        self.SingleBillBox = QCheckBox(self.centralwidget)
        self.SingleBillBox.setObjectName(u"SingleBillBox")
        self.SingleBillBox.setGeometry(QRect(40, 40, 161, 26))
        MainWindow.setCentralWidget(self.centralwidget)
        self.menubar = QMenuBar(MainWindow)
        self.menubar.setObjectName(u"menubar")
        self.menubar.setGeometry(QRect(0, 0, 849, 25))
        self.menuMAS_Reader = QMenu(self.menubar)
        self.menuMAS_Reader.setObjectName(u"menuMAS_Reader")
        MainWindow.setMenuBar(self.menubar)
        self.statusbar = QStatusBar(MainWindow)
        self.statusbar.setObjectName(u"statusbar")
        MainWindow.setStatusBar(self.statusbar)

        self.menubar.addAction(self.menuMAS_Reader.menuAction())
        self.menuMAS_Reader.addAction(self.actionDatei)

        self.retranslateUi(MainWindow)

        QMetaObject.connectSlotsByName(MainWindow)
    # setupUi

    def retranslateUi(self, MainWindow):
        MainWindow.setWindowTitle(QCoreApplication.translate("MainWindow", u"MainWindow", None))
        self.actionDatei.setText(QCoreApplication.translate("MainWindow", u"Datei", None))
        self.ExcelBox.setText(QCoreApplication.translate("MainWindow", u"Excel Export", None))
        self.PDFBox.setText(QCoreApplication.translate("MainWindow", u"PDF Export", None))
        self.RemoveButton.setText(QCoreApplication.translate("MainWindow", u"Verschieben (Doppelte \u00dcberschreiben)", None))
        self.CopyButton.setText(QCoreApplication.translate("MainWindow", u"Kopieren", None))
        self.IndexButton.setText(QCoreApplication.translate("MainWindow", u"Nur Indizieren", None))
        ___qtreewidgetitem = self.treeWidget.headerItem()
        ___qtreewidgetitem.setText(1, QCoreApplication.translate("MainWindow", u"Rechnungen", None));
        ___qtreewidgetitem.setText(0, QCoreApplication.translate("MainWindow", u"Aufstellorte", None));
        self.StartButton.setText(QCoreApplication.translate("MainWindow", u"Start", None))
        self.PathBox.setHtml(QCoreApplication.translate("MainWindow", u"<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.0//EN\" \"http://www.w3.org/TR/REC-html40/strict.dtd\">\n"
"<html><head><meta name=\"qrichtext\" content=\"1\" /><meta charset=\"utf-8\" /><style type=\"text/css\">\n"
"p, li { white-space: pre-wrap; }\n"
"</style></head><body style=\" font-family:'Segoe UI'; font-size:9pt; font-weight:400; font-style:normal;\">\n"
"<p style=\" margin-top:0px; margin-bottom:0px; margin-left:0px; margin-right:0px; -qt-block-indent:0; text-indent:0px;\"><span style=\" font-size:12pt;\">Input</span></p></body></html>", None))
        self.SingleBillBox.setText(QCoreApplication.translate("MainWindow", u"Einzelne Rechnung", None))
        self.menuMAS_Reader.setTitle(QCoreApplication.translate("MainWindow", u"MAS Reader", None))
    # retranslateUi

