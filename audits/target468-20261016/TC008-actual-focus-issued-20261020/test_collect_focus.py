"""Pure local request metadata format only; no provider calls."""
import unittest
from collect_focus import versioned_request,unique_pairs
class Format(unittest.TestCase):
 def test_actual_v1_label_and_duplicate_metadata_rejected(self):
  class Request:pass
  q=versioned_request(Request,'owner','kernel',1,page_size=100,page_token='next')
  self.assertEqual(vars(q),{'user_name':'owner','kernel_slug':'kernel','version_label':'v1','page_size':100,'page_token':'next'})
  for bad in [True,0,2,'1']:
   with self.assertRaises(AssertionError):versioned_request(Request,'owner','kernel',bad)
  with self.assertRaises(AssertionError):unique_pairs([('x',1),('x',2)])
if __name__=='__main__':unittest.main()
