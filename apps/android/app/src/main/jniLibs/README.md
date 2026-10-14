Сюда cargo-ndk кладёт скомпилированные библиотеки:
  app/src/main/jniLibs/arm64-v8a/libkarta.so
  app/src/main/jniLibs/x86_64/libkarta.so

Сборка:
  cargo ndk -t arm64-v8a -t x86_64 -o app/src/main/jniLibs build
