# This script checks whether trusted code is in the src/tcb folder
# This isn't meant to really handle adverserial input, its more like a 
# best-practices linter

# There are three possible sources of trusted code
# 1. the [trusted] annotation
# 2. the effect! macro
# 3. the assume! macro

# This script just greps for these three sources of trusted code

# Grep flags used:
# -l flag = list file names instead of lines found
# -R flag = recursive

# TODO: this should probably exclude trusted/effect/assume/unsafe 
# anotations that are in comments

# Check for [trusted] annotation
check_for_trusted=`grep -lR "\[trusted\]" src | grep -v "src\/tcb" | grep -v "src\/tests" | grep -v "src\/stats"`
if [ -z "$check_for_trusted" ]
then 
	result1=1
else
	result1=0
	echo "Found [trusted] annotations outside the src/tcb folder"
	grep -lR "\[trusted\]" src | grep -v "src\/tcb" | grep -v "src\/tests" | grep -v "src\/stats"
fi

# 2. Check for effect! macro
check_for_effect=`grep -lR "\sdo_effect!" src | grep -v "src\/tcb" | grep -v "src\/tests" | grep -v "src\/stats"`
if [ -z "$check_for_effect" ]
then 
	result2=1
else
	result2=0
	echo "Found do_effect! annotations outside the src/tcb folder"
	grep -lR "\sdo_effect!" src | grep -v "src\/tcb" | grep -v "src\/tests" | grep -v "src\/stats"
fi

# 3. Check for assume! macro
check_for_assume=`grep -lR "\sassume!" src | grep -v "src\/tcb" | grep -v "src\/tests" | grep -v "src\/stats"`
if [ -z "$check_for_assume" ]
then
	result3=1
else
	result3=0
	echo "Found assume! annotations outside the src/tcb folder"
	grep -lR "\sassume!" src | grep -v "src\/tcb" | grep -v "src\/tests" | grep -v "src\/stats"
fi

# 3. Check for unsafe annotations
check_for_unsafe=`grep -lR "\sunsafe\s" src | grep -v "src\/tcb" | grep -v "src\/tests" | grep -v "src\/stats"`
if [ -z "$check_for_unsafe" ]
then
	result4=1
else
	result4=0
	echo "Found unsafe annotations outside the src/tcb folder"
	grep -lR "\sunsafe\s" src | grep -v "src\/tcb" | grep -v "src\/tests" | grep -v "src\/stats"
fi

# If all tests passed, let the user know
if [ "$result1" == "1" -a "$result2" == "1" -a "$result3" == "1" -a "$result4" == "1" ]
then 
	echo "All trusted code is in src/tcb. Good job!"
fi

