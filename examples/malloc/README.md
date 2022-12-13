To use this example, run the following commands (adjusted for your system)

git clone https://github.com/bminor/glibc glibc
{REPLACE malloc.c with my instrumented malloc.c or add your own annotations}
mkdir glibcbuild
cd glibcbuild
mkdir install
export glibc_install="$(pwd)/install"
../glibc/configure --prefix "$glibc_install" CFLAGS="-g3 -O2"
make -j `nproc`
make install -j `nproc`
cd install
{COPY IN build.sh}
{COPY IN test_malloc.c}
./test_glibc.sh test_malloc
./test_malloc.out


Then record that.


NOTE:

You must compile glibc with O2 or O3. It will not compile without it. Also, if you make changes to malloc.c you must recompile glibc. This seems obvious but I accidentally forgot once and it really confused me.
