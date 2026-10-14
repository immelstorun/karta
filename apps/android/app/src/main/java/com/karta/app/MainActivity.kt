package com.karta.app

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    companion object {
        init {
            // Загружаем нативную библиотеку karta
            System.loadLibrary("karta")
        }
    }

    // Нативная функция — точка входа в egui-приложение.
    // JNI-символ: Java_com_karta_app_MainActivity_karta_1android_1start
    private external fun karta_android_start()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Запуск egui-приложения
        karta_android_start()
    }
}
