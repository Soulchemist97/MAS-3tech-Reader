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
    QMenu, QMenuBar, QRadioButton, QSizePolicy,
    QStatusBar, QTreeWidget, QTreeWidgetItem, QWidget)

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
        self.ExcelBox.setGeometry(QRect(40, 280, 93, 26))
        self.PDFBox = QCheckBox(self.centralwidget)
        self.PDFBox.setObjectName(u"PDFBox")
        self.PDFBox.setGeometry(QRect(40, 310, 101, 26))
        self.radioButton = QRadioButton(self.centralwidget)
        self.radioButton.setObjectName(u"radioButton")
        self.radioButton.setGeometry(QRect(40, 210, 301, 21))
        self.radioButton_2 = QRadioButton(self.centralwidget)
        self.radioButton_2.setObjectName(u"radioButton_2")
        self.radioButton_2.setGeometry(QRect(40, 240, 112, 26))
        self.radioButton_3 = QRadioButton(self.centralwidget)
        self.radioButton_3.setObjectName(u"radioButton_3")
        self.radioButton_3.setGeometry(QRect(40, 170, 131, 26))
        self.treeWidget = QTreeWidget(self.centralwidget)
        self.treeWidget.setObjectName(u"treeWidget")
        self.treeWidget.setGeometry(QRect(360, 50, 461, 411))
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
        self.ExcelBox.setText(QCoreApplication.translate("MainWindow", u"Excel", None))
        self.PDFBox.setText(QCoreApplication.translate("MainWindow", u"PDF Export", None))
        self.radioButton.setText(QCoreApplication.translate("MainWindow", u"Verschieben (Doppelte \u00dcberschreiben)", None))
        self.radioButton_2.setText(QCoreApplication.translate("MainWindow", u"Kopieren", None))
        self.radioButton_3.setText(QCoreApplication.translate("MainWindow", u"Nur Indizieren", None))
        ___qtreewidgetitem = self.treeWidget.headerItem()
        ___qtreewidgetitem.setText(1, QCoreApplication.translate("MainWindow", u"Rechnungen", None));
        ___qtreewidgetitem.setText(0, QCoreApplication.translate("MainWindow", u"Aufstellorte", None));
        self.menuMAS_Reader.setTitle(QCoreApplication.translate("MainWindow", u"MAS Reader", None))
    # retranslateUi

