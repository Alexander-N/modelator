// The modules with their methods below are provided by us. 
//
// We can mix as we wish the methods dules that are implemented in Jsonnet itself
// with the methods that are implemented externally in Rust.
//
// Methods that are implemented in Jsonnet will simply do their job.
// For methods that are implemented externally, a call graph will be generated.
//
// In the below example, only the tlc.test method is implemented in Jsonnet,
// the rest is delegated to Rust. This is only an example, 
// in reality many more will be implemented in Jsonnet.


local tla = {
  instantiate_model(model, config): {
    call: "tla.instantiate_model",
    args: [model, config],
  },
  from_file(filename): {
    call: "tla.from_file",
    args: [filename],
  },
  operator(model, name): {
    call: "tla.operator",
    args: [name],
  },
  negate(assertion): {
    call: "tla.negate",
    args: [assertion],
  },

};

local tlaconfig = {
  from_file(filename): {
    call: "tlaconfig.from_file",
    args: [filename],
  },

};

local tlc = {
  local tlc = self,

  check(model, assertion): {
    call: "tlc.check",
    args: [model, assertion], 
  },

  test(model, config, assertion): 
    local instance = tla.instantiate_model(model, config);
    tlc.check(instance, tla.negate(assertion)),
};


// This is the configuration of the test that will reside in the user's repo
// What it produces, is a call graph to be executed by modelator.

local model = tla.from_file("IBC.tla");
local config = tlaconfig.from_file("IBC.cfg");
local tests = tla.from_file("IBCTests.tla");
local test = tla.operator(tests, "ICS02CreateOKTest");

tlc.test(model, config, test)

/* This is the result that will be processed by modelator

{
   "args": [
      {
         "args": [
            {
               "args": [
                  "IBC.tla"
               ],
               "call": "tla.from_file"
            },
            {
               "args": [
                  "IBC.cfg"
               ],
               "call": "tlaconfig.from_file"
            }
         ],
         "call": "tla.instantiate_model"
      },
      {
         "args": [
            {
               "args": [
                  "ICS02CreateOKTest"
               ],
               "call": "tla.operator"
            }
         ],
         "call": "tla.negate"
      }
   ],
   "call": "tlc.check"
}
*/