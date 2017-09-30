#!/usr/bin/zsh

#Some boilerplate
function assert() {
    if [[ $1 == $2 ]]; then
        echo "Content is the same, all is well";
    else
        echo "Assertion failed content: "$1" is not the same as content "$2""
        kill -9 $3;
        exit 1;
    fi
}

#Get in the main dir
cd .. &&

#Set up the test dirs
rm -rf test_dir_1 &&
rm -rf test_dir_2 &&
mkdir test_dir_1 &&
mkdir test_dir_2 &&
chmod -R 777 * &&

#Start process
cargo run 'test.config.json' &;
sleep 5s;
PROC_ID=$! &&

#Create some files and directores
mkdir test_dir_1/A &&
sleep 0.3s
mkdir test_dir_1/B &&
sleep 0.3s
mkdir test_dir_1/C &&
sleep 0.3s
echo "BBBBBB" > test_dir_1/a &&
sleep 0.3s
mv test_dir_1/a test_dir_1/b &&
sleep 0.3s
mkdir test_dir_1/C/A &&
sleep 0.3s
mkdir test_dir_1/C/B &&
sleep 0.3s
rm -r test_dir_1/C/A &&
sleep 0.3s
echo "AAAAAA" > test_dir_1/C/B/a &&
sleep 0.3s
chmod -R 777 * &&

#Test the sync
assert $(cat test_dir_1/b), $(cat test_dir_2/b), $PROC_ID;
#assert $(ls test_dir_1/),   $(ls test_dir_2/), $PROC_ID; <--- find how to capture ls out corr
#assert $(ls test_dir_1/A),  $(ls test_dir_2/A), $PROC_ID;
assert $(cat test_dir_1/C/a), $(cat test_dir_2/C/a), $PROC_ID;
#Clean up
kill -9 $PROC_ID;
