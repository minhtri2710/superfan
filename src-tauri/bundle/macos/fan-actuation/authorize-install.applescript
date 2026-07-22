on run argv
    if (count of argv) is not 1 then error "Expected installer path"
    do shell script quoted form of (item 1 of argv) with administrator privileges
end run
