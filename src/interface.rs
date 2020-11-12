pub trait ObjectManager{

}


pub trait ObjectBase{
    fn write(o:&dyn ObjectManager);
    fn read(o:&dyn ObjectManager);
    fn get_typeid()->u16;
}