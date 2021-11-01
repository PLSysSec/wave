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

# Check for [trusted] annotation
check_for_trusted=`grep -lR "\[trusted\]" src | grep -v "src\/tcb"`
if [ -z "$check_for_trusted" ]
then 
	result1=1
else
	result1=0
	echo "Found [trusted] annotations outside the src/tcb folder"
	grep -lR "\[trusted\]" src | grep -v "src\/tcb"
fi

# 2. Check for effect! macro
check_for_effect=`grep -lR "\seffect!" src | grep -v "src\/tcb"`
if [ -z "$check_for_effect" ]
then 
	result2=1
else
	result2=0
	echo "Found effect! annotations outside the src/tcb folder"
	grep -lR "\seffect!" src | grep -v "src\/tcb"
fi

# 3. Check for assume! macro
check_for_assume=`grep -lR "\sassume!" src | grep -v "src\/tcb"`
if [ -z "$check_for_assume" ]
then
	result3=1
else
	result3=0
	echo "Found assume! annotations outside the src/tcb folder"
	grep -lR "\sassume!" src | grep -v "src\/tcb"
fi


# If all tests passed, let the user know
if [ "$result1" == "0" -a "$result2" == "0" -a "$result3" == "1" ]
then 
	echo "All trusted code is in src/tcb. Good job!"
fi

