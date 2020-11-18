/**
encode时：按照二分算法查找根据字符的code值判断是否需要encode，然后挑选最靠前的一个
decode时：复制一份的数据，按照归并排序将数据排序为("&xxx", 0xxx, 'x', len)的元组数组
用HashMap<(char, bool), usize>：存取找到的首字母开始和结束索引，缩小查找范围
查找的时候，先找到首字母的首位端，再二分查找，根据len，第二个字母的charCode
*/