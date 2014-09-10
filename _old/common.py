from collections import namedtuple
from struct import Struct

class Proxy(object):
	"""

	Taken from http://code.activestate.com/recipes/496741/
	"""

	__slots__ = ["_obj", "__weakref__"]
	def __init__(self, obj):
		object.__setattr__(self, "_obj", obj)

	#
	# proxying (special cases)
	#
	def __getattribute__(self, name):
		return getattr(object.__getattribute__(self, "_obj"), name)
	def __delattr__(self, name):
		delattr(object.__getattribute__(self, "_obj"), name)
	def __setattr__(self, name, value):
		setattr(object.__getattribute__(self, "_obj"), name, value)

	def __nonzero__(self):
		return bool(object.__getattribute__(self, "_obj"))
	def __str__(self):
		return str(object.__getattribute__(self, "_obj"))
	def __repr__(self):
		return repr(object.__getattribute__(self, "_obj"))

	#
	# factories
	#
	_special_names = [
		'__abs__', '__add__', '__and__', '__call__', '__cmp__', '__coerce__',
		'__contains__', '__delitem__', '__delslice__', '__div__', '__divmod__',
		'__eq__', '__float__', '__floordiv__', '__ge__', '__getitem__',
		'__getslice__', '__gt__', '__hash__', '__hex__', '__iadd__', '__iand__',
		'__idiv__', '__idivmod__', '__ifloordiv__', '__ilshift__', '__imod__',
		'__imul__', '__int__', '__invert__', '__ior__', '__ipow__', '__irshift__',
		'__isub__', '__iter__', '__itruediv__', '__ixor__', '__le__', '__len__',
		'__long__', '__lshift__', '__lt__', '__mod__', '__mul__', '__ne__',
		'__neg__', '__oct__', '__or__', '__pos__', '__pow__', '__radd__',
		'__rand__', '__rdiv__', '__rdivmod__', '__reduce__', '__reduce_ex__',
		'__repr__', '__reversed__', '__rfloorfiv__', '__rlshift__', '__rmod__',
		'__rmul__', '__ror__', '__rpow__', '__rrshift__', '__rshift__', '__rsub__',
		'__rtruediv__', '__rxor__', '__setitem__', '__setslice__', '__sub__',
		'__truediv__', '__xor__', 'next',
	]

	@classmethod
	def _create_class_proxy(cls, theclass):
		"""creates a proxy for the given class"""

		def make_method(name):
			def method(self, *args, **kw):
				return getattr(object.__getattribute__(self, "_obj"), name)(*args, **kw)
			return method

		namespace = {}
		for name in cls._special_names:
			if hasattr(theclass, name):
				namespace[name] = make_method(name)
		return type("%s(%s)" % (cls.__name__, theclass.__name__), (cls,), namespace)

	def __new__(cls, obj, *args, **kwargs):
		"""
		creates an proxy instance referencing `obj`. (obj, *args, **kwargs) are
		passed to this class' __init__, so deriving classes can define an
		__init__ method of their own.
		note: _class_proxy_cache is unique per deriving class (each deriving
		class must hold its own cache)
		"""
		try:
			cache = cls.__dict__["_class_proxy_cache"]
		except KeyError:
			cls._class_proxy_cache = cache = {}
		try:
			theclass = cache[obj.__class__]
		except KeyError:
			cache[obj.__class__] = theclass = cls._create_class_proxy(obj.__class__)
		ins = object.__new__(theclass)
		theclass.__init__(ins, obj, *args, **kwargs)
		return ins

class ProxyNoClose(Proxy):
	def close(self):
		pass

class cached_property(object):
	"""
	Decorator that converts a method with a single self argument into a
	property cached on the instance.

	Taken from https://github.com/django/django/blob/1.6/django/utils/functional.py#L38
	"""
	def __init__(self, func):
		self.func = func

	def __get__(self, instance, type=None):
		if instance is None:
			return self
		res = instance.__dict__[self.func.__name__] = self.func(instance)
		return res

def file_chunks(file, count=2**15):
	return iter((lambda:file.read(count)), b'')

def namedstruct(name, pack_fmt, fields):
	struct = Struct(pack_fmt)

	class Class(namedtuple(name, fields)):
		_name = name
		_pack_fmt = pack_fmt
		_struct = struct
		_size = struct.size

		@classmethod
		def _unpack(cls, buffer):
			return cls(*cls._struct.unpack(buffer))

		def _pack(self):
			return self._struct.pack(*self)


	Class.__name__ = name
	Class.__qualname__ = name
	return Class 
