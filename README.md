# c2rs-img
Convert C image arrays created with GIMP to Rust

-Usage

Export image as a C source file from GIMP using File > Export As, then selecting the .c file extension.
In the dialog window that opens, leave the "Prefixed name:" field as the default value "gimp_image", and check
the "Use Glib types" and "Save Alpha Channel" options.

Execute the c2rs-img command with the file name as the first argument. A custom variable name can be added as the second optional argument:
c2rs-img image.c custom_name

The resulting Rust source file will contain the bitmap data as an array of RGBA u8 values. That is useful for embedded projects with minimal
resources or using with WASM applications.
