---
layout: post
title:  "Compiling Rust on CentOS 5.10"
date:   2014-12-27
categories: rust ops
---

> You do not need to compile Rust for CentOS yourself, the nightly works
> just fine, [because wonderful Rust devs already did that for you][github-stdlib-centos-issue].
> However, if for some weird reason you really, really need it, well, I hope this
> guide might save you some time.

When configuring rust for compilation on CentOS 5.10, you will be greeted with
this message:

    The selected GCC C++ compiler is not new enough to build LLVM. Please upgrade
    to GCC 4.7.

So, first step is to get more up-to-date gcc. We can compile our own.

# Compiling GCC

Go to `https://gcc.gnu.org` and [get latest gcc url][gcc-mirrors], unpack it.

{% highlight bash %}
wget http://www.netgull.com/gcc/releases/gcc-4.9.2/gcc-4.9.2.tar.gz
tar -xzf gcc-4.9.2.tar.gz
{% endhighlight %}

The [GCC download page manual][gcc-download-doc] says that we will also need
`gmp`, `mpfr` and `mpc` libraries. They are conveniently listed in
[GCC `infrastructure` dir][gcc-mirror-infrastructure] on any mirror.

Download them, extract them, move them into the gcc source dir:

{% highlight bash %}
wget http://www.netgull.com/gcc/infrastructure/gmp-4.3.2.tar.bz2
wget http://www.netgull.com/gcc/infrastructure/mpfr-2.4.2.tar.bz2
wget http://www.netgull.com/gcc/infrastructure/mpc-0.8.1.tar.gz
tar -xjf gmp-4.3.2.tar.bz2
tar -xjf mpfr-2.4.2.tar.bz2
tar -xzf mpc-0.8.1.tar.gz
mv gmp-4.3.2 gcc-4.9.2/gmp
mv mpfr-2.4.2 gcc-4.9.2/mpfr
mv mpc-0.8.1 gcc-4.9.2/mpc
{% endhighlight %}

We are going to install gcc into /opt/gcc-4.9.2, so, let's configure for that:

{% highlight bash %}
cd gcc-4.9.2
./configure --prefix=/opt/gcc-4.9.2
{% endhighlight %}

Compile and then install (this will take forever and 2 minutes):

{% highlight bash %}
make
make install
{% endhighlight %}

When done, the `/opt/gcc-4.9.2/bin/gcc --version` command should say:

    gcc (GCC) 4.9.2

# Compiling Rust

Rust will need python version > 2.5. I had EPEL repositories enabled,
so this was not a big deal. If you don't,
[you might want to enable them too][epel-repos].

And then install `python26`:
{% highlight bash %}
yum install python26
{% endhighlight %}

You can then symlink Python 2.6 as default:
{% highlight bash %}
ln -sf /usr/bin/python2.6 /usr/bin/python
{% endhighlight %}

The `python -V` should print:

{% highlight bash %}
Python 2.6.8
{% endhighlight %}

Change directory to your downloaded or cloned Rust source.

{% highlight bash %}
cd ~/rust
{% endhighlight %}

Before running `./configure`, we need to make the script use
our new GCC instead of system default. This tricky part is the reason this
guide exists. Override the environment like this:

{% highlight bash %}
export PATH=/opt/gcc-4.9.2/bin/:$PATH
export LD_LIBRARY_PATH=/opt/gcc-4.9.2/lib/
export LD_PRELOAD='/opt/gcc-4.9.2/lib64/libstdc++.so.6'
{% endhighlight %}

Then, run the `./configure` (we will install rust to `/opt/rust`):

{% highlight bash %}
./configure --prefix=/opt/rust
{% endhighlight %}

We are ready to run make. Let's do it:

{% highlight bash %}
make
{% endhighlight %}

However, it may fail to download rustc snapshot using python curl library:

{% highlight bash %}
fetch: x86_64-unknown-linux-gnu/stage0/bin/rustc
determined most recent snapshot: rust-stage0-2014-12-20-8443b09-linux-x86_64-4f3c8b092dd4fe159d6f25a217cf62e0e899b365.tar.bz2

curl: (35) error:14077410:SSL routines:SSL23_GET_SERVER_HELLO:sslv3 alert handshake failure
Traceback (most recent call last):
  File "/root/rust-nightly/src/etc/get-snapshot.py", line 61, in <module>
    get_url_to_file(url, dl)
  File "/root/rust-nightly/src/etc/snapshot.py", line 142, in get_url_to_file
    raise Exception("failed to fetch url")
Exception: failed to fetch url
make: *** [x86_64-unknown-linux-gnu/stage0/bin/rustc] Error 1
{% endhighlight %}

It looks like the server that hosts snapshots and the python curl library
can not agree on encryption protocol. We will need to download the snapshot over
http. Copy and append that long `rust-stage0-2014-...` path to
`http://static.rust-lang.org/stage0-snapshots/` url. The wget command will look
similar to this (we need to download to "dl" dir):

{% highlight bash %}
wget -P dl http://static.rust-lang.org/stage0-snapshots/rust-stage0-2014-12-20-8443b09-linux-x86_64-4f3c8b092dd4fe159d6f25a217cf62e0e899b365.tar.bz2
{% endhighlight %}

When you run make again, it should pickup the downloaded file and compile rust:

{% highlight bash %}
make
make install
{% endhighlight %}

Wait forever and 2 minutes again, and hope for the best.

[github-stdlib-centos-issue]: https://github.com/rust-lang/rust/issues/15365
[gcc-mirrors]: https://gcc.gnu.org/mirrors.html
[gcc-download-doc]: https://gcc.gnu.org/install/download.html
[gcc-mirror-infrastructure]: http://www.netgull.com/gcc/infrastructure/
[epel-repos]: http://www.rackspace.com/knowledge_center/article/install-epel-and-additional-repositories-on-centos-and-red-hat
