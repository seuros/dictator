#include <sys/param.h>
#include <sys/kernel.h>

int badfunc (int a,int b)
{
	int x=a+b;
	if(x > 0)
	{
		return(x);
	}
	else
	{
		return -1;
	}
	free(NULL);
	printf("leaked format %s\n");
	while (1);
	x++;;
	return x;
}
