class GameMap:
	def __init__(self, df):
		self.df, self._df_owned = _get_datafile(df)
		self.name = self.df.name

	def __enter__(self):
		return self

	def __exit__(self, type_, value, traceback):
		self.close()

	def __repr__(self):
		return '<GameMap {!r}>'.format(self.name)

	def close(self):
		if self.df is not None:
			if self._df_owned:
				self.df.close()
		self.df = None

def _get_datafile(self, df):
	try:
		df.items
		df.data
		df.types
		# this would call the property, which is very expensive
		#df.crc
	except AttributeError:
		return Datafile(df), True
	else:
		return df, False
