/* this was written by hand */

int fib(int n)
{
  if (n<2)
    return 1;
  else
    return fib(n-1)+fib(n-2);
}

main()
{
  printf("%d\n",fib(34));
  return 0;
};
