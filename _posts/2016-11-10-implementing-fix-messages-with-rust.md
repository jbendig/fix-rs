---
layout: post
title: Implementing FIX Messages with Rust
date: 2016-11-10 17:06:40
---

In **[fix-rs]({{ site.github.url }}/about/)**, we need a way for users of the library to work with messages that are sent and received over a network. These messages must be serializable and deserializable into different formats. They also must be easy to create, read and modify by code. But most importantly, they must follow various rules defined by the FIX specification. This post covers the thought process used to design an efficient implementation. The result is an approach that is simple to use and maintainable as *fix-rs* grows.

About FIX Messages
------------------

A FIX message is composed as a set of fields. Each field has a tag and a value. The tag is defined as a unique number. The value has a type (string, number, array of bytes, date, etc.) and sometimes a field specific rule used to determine if it is valid.

A field can be used in more than one message. The message determines if the field is optional, required, or only required under various conditions like when another field has a specific value. There are also special fields used to indicate repeating groups of fields -- which is the only time a field can appear more than once in a single message.

![Messages are made up of fields. A single field might be used in more than one message.]({{ site.url }}/images/fields_and_messages.svg)

As of the newest spec, version 5.0 Server Pack 2, there are over 100 messages that share from over 1500 fields. End users can also pre-negotiate private messages, fields, and custom types.

The byte level representation of a message depends on the underlying transport protocol and is beyond the scope of this post. If you want to know more, see the spec (available free after registration) on the [FIX Protocol](http://www.fixtradingcommunity.org/) website.

The HashMap Approach
--------------------

The most obvious way to store a message is with a [HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html). This makes sense because a message is made up of fields with unique tags. The tag can be the HashMap's key with a string, integer, or an array of bytes as its type -- whichever happens to be more convenient or performant in practice.

The value is a little more troublesome because its type varies by field. Further complicating things, this type could be repeating groups of fields that can be nested. Let's simplify the value types to just a string, an array of bytes, and an array of groups. This way we can see how well this approach works before investing more effort into supporting things like custom types.

Thanks to a convenient property of enums in Rust, where data can be associated with each value, the entire implementation is simple:

```rust
enum MessageValue {
    String(String),
    Data(Vec<u8>),
    Groups(Vec<Message>),
}

type HashMap<String,MessageValue> Message;
```

Building a message then looks like this:

```rust
let mut groups = Vec::new();
let group = Message::new();
group["372"] = MessageValue::String("A"); //String
group["385"] = MessageValue::String("S"); //Char
groups.push(group);

let mut message: Message::new();
message["96"] = MessageValue::Data(b"Some\x00\x01Data\xff".to_vec());
message["108"] = MessageValue::String("30"); //Integer
message["384"] = MessageValue::Groups(groups); //Groups of fields
message["553"] = MessageValue::String("User"); //String
```

And this is how you would read a message:

```rust
//Read a single field.
if let Some(message_value) = message.get("108") {
    if let MessageValue::String(string_value) = message_value {
        //Now make sure string is valid for this field before using it.
    }
    else {
        //Wrong type.
    }
}
else {
    //Not found, was it required?
}

//Read a field inside a repeating group.
if let Some(message_value) = message.get("384") {
    if let MessageValue::Groups(groups) = message_value {
        for group in &groups {
            if let Some(group_value) =  group.get("372") {
                if let MessageValue::String(string_value) {
                    //Now make sure string is valid for this field before
                    //using it.
                }
            }
            else {
                //Not found, was it required?
            }

            //TODO: Don't forget all of the other fields!
        }
    }
    else {
        //Wrong type.
    }
}
else {
    //Not found, was it required?
}

... //Other fields are left out for brevity.
```

This turns out to be an awkward approach with a lot of problems:

  - No type checking when building a message.
  - Lots of redundant error checking when reading a message. Especially considering fields are often shared between messages.
  - Magic numbers are everywhere.
  - Accessing values in a hash map is (relatively) expensive.
  - No way to describe which fields are required and which are not.
  - No way to describe any required ordering of fields.

We could push on and create lots of little functions to handle things like message construction and field error checking. But, this would greatly increase the complexity of using the library. It would also be difficult to maintain considering the large amount of messages and fields. Time for a more structured approach.

A Usability Point of View
-------------------------

Let's find out what would be ideal from a usability perspective. It should be easy to work with and it should be hard to use incorrectly. Maybe a simple `struct`? Each field could be represented by a variable of an appropriate type. The user could access fields directly and the compiler would automatically enforce correctness.

```rust
struct MsgTypeGrp {
    ref_msg_type: String,
    msg_direction: u8
    ...
}

struct Logon {
    raw_data: Vec<u8>,
    heart_bt_int: i32,
    msg_type_grp: Vec<MsgTypeGrp>,
    username: String,
    ...
}
```

There's a number of problems that need to be solved to make this work on the library side though. First off, how do we know which fields are required? We could wrap the field type in an [`Option<T>`](https://doc.rust-lang.org/std/option/enum.Option.html), but many fields are content being set to a default or just left empty. Some other issues are:

  - How do we associate a tag with a field?
  - How do we serialize or deserialize these?
  - In the case of serializing, how do we construct the correct one dynamically and then set values to the correct fields?
  - How do we specify when there is a required orderings of fields?
  - How do we specify fields and their properties consistently when they are used by more than one message?

Hopefully we can find the right solutions so users can work directly with structs like these.

The Procedural Macros 1.1 Approach
----------------------------------

We could try to take advantage of the upcoming [Procedural Macros 1.1](https://github.com/rust-lang/rfcs/pull/1681) feature. This is the way libraries like [Serde](https://github.com/serde-rs/serde) handle serialization. The procedural macro would generate all of the implementation details automatically for any `struct` prefixed with our custom derive (ie. `#[derive(Message)]`). Defining a message might look something like this:

```rust
//Rules for how the field must be used when parsed or serialized.
enum Rule {
    Nothing,
    ConfirmPreviousTag{ previous_tag: &'static [u8]) },
    ...
}

#[derive(MessageGroup)]
struct MsgTypeGrp {
    #[fix(tag=b"372",required=true)]
    ref_msg_type: String,
    #[fix(tag=b"385",required=true)]
    msg_direction: u8,
    ...
}

#[derive(Message)]
struct Logon {
    #[fix(tag=b"96",
          required=false,
          rule=Rule::ConfirmPreviousTag{ previous_tag: "95" })]
    raw_data: Vec<u8>,
    #[fix(tag=b"108",required=true)]
    heart_bt_int: i32,
    #[fix(tag=b"384",required=false)]
    msg_type_grp: Vec<MsgTypeGrp>,
    #[fix(tag=b"553",required=false)]
    username: String,
    ...
}
```

Everything needed to describe a message is here, but specifying the properties for each field is very redundant and hard to scan visually. For example, specifying requirements like `Rule::ConfirmPreviousTag`, which defines an ordering, could easily be forgotten. Furthermore, the feature is still unstable as of Rust 1.12. Let's keep looking.

The macro_rules! Approach
-------------------------

Building on the previous approach, it seems that the fields should be specified separately for consistency. Then messages themselves can be composed from these field definitions. Due to the large amount of redundancy expected, it seems like macros (the stable kind) could be a fit here.

```rust
define_fields!(
    RawDataLength: PhantomData<()> = b"95"
        => Rule::PrepareForBytes{ bytes_tag: RawData::tag() },
    RawData: Vec<u8> = b"96"
        => Rule::ConfirmPreviousTag{ previous_tag: RawDataLength::tag() },
    HeartBtInt: i32 = b"108",
    RefMsgType: String = b"372",
    NoMsgTypeGrp: Vec<Box<MsgTypeGrp>> = b"384",
    MsgDirection: u8 = b"385",
    ...
);

define_message_group!(MsgTypeGrp {
    REQUIRED, ref_msg_type: RefMsgType,
    REQUIRED, msg_direction: MsgDirection,
    ...
});

define_message!(Logon {
    NOT_REQUIRED, raw_data_length: RawDataLength,
    NOT_REQUIRED, raw_data: RawData,
    REQUIRED, heart_bt_int: HeartBtInt,
    NOT_REQUIRED, msg_type_grp: NoMsgTypeGrp,
    NOT_REQUIRED, username: Username,
    ...
});
```

This turns out to be much more readable and it appears easier to maintain because a field only has to be modified in one place. Though, there are a few minor stand out parts.

First, the `RawDataLength` field has to be specified even though it doesn't provide any data. This acts as a hint so the parser knows how many bytes to read into the `RawData` field. The [PhantomData<()>](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) allows us to declare this field without taking up any memory. It also prevents this field from being used accidentally.

The second is the redundancy of the field names in the `define_message!()` macro. It would be nice if we could specify just the field type. Then the macro could perform a conversion from CamelCase to snake_case (`RawData` -> `raw_data`) and stick the result in as the field name. Unfortunately, this can't be done because this type of macro is [hygienic](https://doc.rust-lang.org/book/macros.html#hygiene). The advantage would be an enforced consistency for users referencing the FIX spec. They should always know how to access a field without cross-referencing our documentation.

While not critical, I suspect these two could be handled using some Procedural Macros 1.1 cleverness once it's stable.

The `REQUIRED` and `NOT_REQUIRED` prefixes indicate which fields must be present when a message is deserialized. The exact representation of these prefixes depends on the bodies of the `define_message!()` and `define_message_group!()` macros. However, since there are only two choices, we can expect these to just be bool constants. Another option is to use an `enum` for safety so the compiler will require a specific value instead of an expression. But, I personally think it makes the code harder to read due to the added visual noise.

```rust
enum Required {
    Yes,
    No
}

define_message!(Logon {
    Required::No, raw_data_length: RawDataLength,
    Required::No, raw_data: RawData,
    Required::Yes, heart_bt_int: HeartBtInt,
    Required::No, msg_type_grp: NoMsgTypeGrp,
    Required::No, username: Username,
    ...
})
```

### Designing the Macro Expansion ###

The easiest way to write a macro is to work on its expansion first and then generalize into a macro body later. Let's start by defining a `trait` that exposes the functionality we need out of a message. Then each message can be a `struct` that implements this `trait`. To simplify the design and support nested repeating groups, a group of fields will be defined using the same `trait`.

```rust
pub trait Message {
    //Tell the parser which field must be first. Needed for repeating group
    //messages only.
    fn first_field(&self) -> &'static [u8];

    //Give the parser a set of all of the possible fields in this message and
    //instructions on how to handle them.
    fn fields(&self) -> HashMap<&'static [u8],Rule>;

    //Give the parser a set of fields that must be specified or else the
    //message is incomplete and should be rejected.
    fn required_fields(&self) -> HashSet<&'static [u8]>;

    //Dynamically set the value of a field by tag. The implementation will be
    //in charge of making sure the bytes meet the type requirements and
    //assigning them to the underlying variable.
    fn set_value(&mut self,tag: &[u8],value: &[u8]) -> bool;

    //Dynamically assign all of the groups to a field by tag.
    fn set_groups(&mut self,tag: &[u8],groups: &[Box<Message>]) -> bool;

    //Convert to `Any` so we can downcast from a Message trait object to the
    //actual struct.
    fn as_any(&self) -> &Any;

    //Construct a new instance into a Box so we can dynamically build a Message.
    fn new_into_box(&self) -> Box<Message>;

    //Serialize the message into a series of bytes for some underlying protocol.
    //Loosly based off of the Read trait.
    fn read_body(&self,buf: &mut Vec<u8>) -> usize;

    //Same as read_body() except it wraps the body with meta information
    //required by the outer most message. (eg. length and checksum).
    fn read(&self,buf: &mut Vec<u8>) -> usize {
        ...
    }
}
```

Filling out this `trait` is fairly straight forward and looks something like below. Note that this won't compile yet.

```rust
#[derive(Clone,Default)]
struct MsgTypeGrp {
    ref_msg_type: String,
    msg_direction: u8,
    ...
}

impl MsgTypeGrp {
    fn new() -> Self {
        MsgTypeGrp {
            ref_msg_type: Default::default(),
            msg_direction: Default::default(),
			...
        }
    }
}

impl Message for MsgTypeGrp {
    ... //Very similar to "impl Message for Logon". See below.
}

#[derive(Clone,Default)]
struct Logon {
    raw_data_length: PhantomType<()>,
    raw_data: Vec<u8>,
    heart_bt_int: i32,
    msg_type_grp: Vec<Box<MsgTypeGrp>>,
    username: String,
    ...
}

impl Logon {
    fn new() -> Self {
        Logon {
            raw_data_length: Default::default(),
            raw_data: Default::default(),
            heart_bt_int: Default::default(),
            msg_type_grp: Default::default(),
            username: Default::default(),
            ...
        }
    }
}

impl Message for Logon {
    fn first_field(&self) -> &'static [u8] {
        RawDataLength::tag()
    }

    fn fields(&self) -> HashMap<&'static [u8],Rule> {
        let mut result = HashMap::new();
        result.insert(RawDataLength::tag(),RawDataLength::rule());
        result.insert(RawData::tag(),RawData::rule());
        result.insert(HeartBtInt::tag(),HeartBtInt::rule());
        result.insert(NoMsgTypeGrp::tag(),NoMsgTypeGrp::rule());
        result.insert(Username::tag(),Username::rule());
        ...

        result
    }

    fn required_fields(&self) -> HashSet<&'static [u8]> {
        let mut result = HashSet::new();
        result.insert(HeartBtInt::Tag());
        ...

        result
    }

    fn set_value&mut self,tag: &[u8],value: &[u8]) -> bool {
        if tag == RawData::tag() {
            self.raw_data.set_value(value)
        }
        else if tag == HeartBtInt::tag() {
            self.heart_bt_int.set_value(value)
        }
        else if tag == Username::tag() {
            self.username.set_value(value)
        }
        ...
        else {
            false
        }
    }

    fn set_groups(&mut self,tag: &[u8],groups: &[Box<Message>]) -> bool {
        if tag == NoMsgTypeGrp::tag() {
            self.msg_type_grp.set_groups(groups)
        }
        ...
        else {
            false
        }
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn new_into_box(&self) -> Box<Message> {
        Box::new(Logon::new())
    }

    fn read_body(&self,buf: &mut Vec<u8>) -> usize {
        let mut byte_count = 0
        byte_count += self.raw_data.read(buf);
        byte_count += self.heart_bt_int.read(buf);
        byte_count += self.msg_type_grp.read(buf);
        byte_count += username.read(buf);
        ...

        byte_count
    }
}
```

Looking at the above expansion, we can see what each field should specify:

  - type: The type of variable in the `struct`.
  - tag: The number representing the field according to the FIX protocol.
  - rule: The rule to follow when serializing or deserializing the field.

This gives a `trait` that looks like:

```rust
pub trait Field {
    type Type;

    fn tag() -> &'static [u8];
    fn rule() -> Rule;
}
```

Which is trivial to implement:

```rust
struct HeartBtInt;

impl Field for HeartBtInt {
    type Type = i32;

    fn tag() -> &'static [u8] {
        b"108"
    }

    fn rule() -> Rule {
        Rule::Nothing
    }
}
```

If we try to compile this, it becomes clear that there is still something missing.

```rust
struct Logon {
    ...
    heart_bt_int: <HeartBtInt as Field>::Type,
    ...
}

impl Message for Logon {
    ...
    fn set_value(&mut self,tag: &[u8],value: &[u8]) -> bool {
        ...
        else if tag == HeartBtInt::tag() {
            self.heart_bt_int.set_value(value) //Compile error!
        }
        ...
    }
    ...
}
```

A compiler error is raised because the functions for working with each field haven't been defined yet. We could try to keep the above syntax by creating a new `trait` that each `Field::Type` must implement. But this might not be flexible enough. For example, what if multiple fields use the same primitive integer but require different ranges of numbers. A better approach is to define a `trait` with an associated type that it acts upon. This way implementations specify the underlying data together with how to do things like assigning a value from an array of bytes.

```rust
pub trait FieldType {
    type Type;

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> bool;
    fn set_groups(field: &mut Self::Type,groups: &[Box<Message>]) -> bool;
    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize;
}

struct IntegerFieldType;

impl FieldType for IntegerFieldType {
    type Type = i32;

    fn set_value(field: &mut Self::Type,bytes:&[u8]) -> bool {
        ... //Convert array of ASCII bytes into integer here.
        true
    }

    fn set_groups(field: &mut Self::Type,groups: &[Box<Message>]) -> bool {
        false //Not a repeating group field.
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        ... //Convert integer into ASCII and append to buf here.
    }
}
```

With some slight modifications to the `Logon` message above, everything now compiles:

```rust
struct HeartBtInt;

impl Field for HeartBtInt {
    type Type = IntegerFieldType;
    ...
}

struct Logon {
    ...
    heart_bt_int: <<HeartBtInt as Field>::Type as FieldType>::Type,
    ...
}

impl Message for Logon {
    ...
    fn set_value(&mut self,tag: &[u8],value: &[u8]) -> bool {
        ...
        else if tag == HeartBtInt::tag() {
            <HeartBtInt as Field>::Type::set_value(&mut self.heart_bt_int,value)
        }
        ...
    }
    ...
}

```

### Implementing the Macros ###

With the target macro expansions working, we can finish by generalizing into actual macros. First up is the `define_fields!()` macro. If you're unfamiliar with the syntax for defining a macro, check out the [Macros](https://doc.rust-lang.org/book/macros.html) chapter of the Rust Book.

```rust
macro_rules! define_fields {
    ( $( $field_name:ident :    //Identifier used to refer to this field.
         $field_type:ty =       //Type of field. Must implement FieldType trait.
         $tag:expr              //Tag of field. Must be a byte string (ie. b"1234").
         $( => $rule:expr)* ),* //Optional rule for how field should be used.
      $(),* ) => { $(           //Optional ',' after final field.
        pub struct $field_name;

        impl Field for $field_name {
            type Type = $field_type;

            fn tag() -> &'static [u8] {
                $tag
            }

            #[allow(unreachable_code)]
            fn rule() -> Rule {
                //Optional rule.
                $(
                    return rule
                )*; //Putting the semicolon outside the expansion makes sure
                    //no more than one rule can be specified.

                //Default rule.
                Rule::Nothing
            }
        }
    )*};
}

//Using the macro is similar to before. The only change is that types are now
//specified using structs which implement the FieldType trait.
define_fields!(
    RawDataLength: NoneFieldType = b"95"
        => Rule::PrepareForBytes{ bytes_tag: RawData::tag() },
    RawData: DataFieldType = b"96"
        => Rule::ConfirmPreviousTag{ previous_tag: RawDataLength::tag() },
    HeartBtInt: IntegerFieldType = b"108",
    RefMsgType: StringFieldType = b"372",
    NoMsgTypeGrp: RepeatingGroupFieldType<MsgTypeGrp> = b"384",
    MsgDirection: CharFieldType = b"385",
    ...
);

```

All pretty straight forward except for a couple of things:

  - `$( => $rule:expr)*`: Allows the rule to be optional. Unfortunately, the `$()*` syntax allows more than one rule to be specified but our `rule()` function only returns one. To work around this, the macro is formatted so the `;` is outside the expansion. This causes the compiler to report an error if more than one rule is specified.
  - `$(),*`: A stylistic choice to optionally allow a comma after the final field. It makes the macro consistent with how structs and enums are often declared in Rust.

Finally, the `define_message!()` macro:

```rust
const REQUIRED: bool = true;
const NOT_REQUIRED: bool = false;

macro_rules! define_message {
    ( $message_name:ident { //Identifier used to refer to this message.
        $( //List of fields that make up this message.
           $field_required:expr, //Boolean representing whether this field is
                                 //required. Use the REQUIRED or NOT_REQUIRED
                                 //constants defined above.
           $field_name:ident :   //Identifier used to refer to this field inside
                                 //the message's struct.
           $field_type:ty        //The type of field as defined using the
                                 //define_fields!() macro.
        ),*
        $(),* //Optional ',' after final field.
      } ) => {
        pub struct $message_name {
            $( pub $field_name: <<$field_type as Field>::Type as FieldType>::Type, )*
        }

        impl $message_name {
            pub fn new() -> Self {
                $message_name {
                    $( $field_name: Default::default(), )*
                }
            }
        }

        impl Message for $message_name {
            #[allow(unreachable_code)]
            fn first_field(&self) -> &'static [u8] {
                $( return { <$field_type as Field>::tag() }; )*

                b""
            }

            fn fields(&self) -> HashMap<&'static [u8],Rule> {
                let mut result = HashMap::new();
                $(
                    result.insert(<$field_type as Field>::tag(),
                                  <$field_type as Field>::rule());
                )*

                result
            }

            fn required_fields(&self) -> HashSet<&'static [u8]> {
                let mut result = HashSet::new();
                $(
                    if $field_required {
                        result.insert(<$field_type as Field>::tag());
                    }
                )*

                result
            }

            fn set_value(&mut self,tag: &[u8],value: &[u8]) -> bool {
                if false {
                    false
                }
                $(
                    else if tag == <$field_type as Field>::tag() {
                        <$field_type as Field>::Type::set_value(&mut self.$field_name,
                                                                value)
                    }
                )*
                else {
                    false
                }
            }

            fn set_groups(&mut self,tag: &[u8],groups: &[Box<Message>]) -> bool {
                if false {
                    false
                }
                $(
                    else if tag == <$field_type as Field>::tag() {
                        <$field_type as Field>::Type::set_groups(&mut self.$field_name,
                                                                 groups)
                    }
                )*
                else {
                    false
                }
            }

            fn as_any(&self) -> &Any {
                self
            }

            fn new_into_box(&self) -> Box<Message> {
                Box::new($message_name::new())
            }

            fn read_body(&self,buf: &mut Vec<u8>) -> usize {
                let mut byte_count: usize = 0;
                $( byte_count += <$field_type as Field>::Type::read(&self.$field_name,
                                                                    buf); )*

                byte_count
            }
        }
    };
}

//Using the macro is the same as before.
define_message!(Logon {
    NOT_REQUIRED, raw_data_length: RawDataLength,
    NOT_REQUIRED, raw_data: RawData,
    REQUIRED, heart_bt_int: HeartBtInt,
    NOT_REQUIRED, msg_type_grp: NoMsgTypeGrp,
    NOT_REQUIRED, username: Username,
    ...
});
```

Summary
-------

Representing a message as a HashMap, where field tags are mapped to values, is an approach that is simple to understand. However, it requires extensive boilerplate code that makes it hard to use and maintain.

With the HashMap approach being too verbose, it made sense to investigate an ideal representation from the viewpoint of a user. This turned out to be just a plain old `struct` for each message. The fields in a FIX message map directly to the fields of a `struct`. With the important benefit that the Rust compiler enforces safety instead of redundant boilerplate.

Finally, an implementation was designed to work around the ideal representation by making extensive use of `macro_rules!`. The end result is a far better solution. It's straight forward to use. The compiler enforces type checking. There is no redundancy. It makes fields consistent. And, it's easy to modify in the future. Score one point for Rust Macros.

### Notes ###

Some code was simplified to keep this post from becoming even longer. See the *fix-rs* commit below for a full implementation.

As of this writing:

  - The most recent version of Rust is [1.12.1](https://blog.rust-lang.org/2016/10/20/Rust-1.12.1.html).
  - The most recent commit of *fix-rs* is [c5d1406](https://github.com/jbendig/fix-rs/tree/c5d140615c9374a79143e86aba5f79d924727192).
