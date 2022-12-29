use tracing::Level;
use tracing_subscriber::Layer;


static VERBOSE_LEVEL: u8 = 0;


pub struct CustomLayer{
    pub verbose_level: u8
}

impl<S> Layer<S> for CustomLayer where S: tracing::Subscriber {
    
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        /*
        println!("Got event!");
        println!("  level={:?}", event.metadata().level());
        println!("  target={:?}", event.metadata().target());
        println!("  name={:?}", event.metadata().name());
        for field in event.fields() {
            println!("  field={}", field.name());
        }
        */
        let mut can_print = false;
        if self.verbose_level <= 0 
            && event.metadata().level().clone() == Level::ERROR
        {
            can_print = true; 
        }else if self.verbose_level == 1 
            && (
                event.metadata().level().clone() == Level::ERROR
                || event.metadata().level().clone() == Level::INFO
            )
        {
            can_print = true;
        }else if self.verbose_level > 1 
            && (
                event.metadata().level().clone() == Level::ERROR
                || event.metadata().level().clone() == Level::INFO
            )
        {
            can_print = true;
        } 

        if can_print {
            print!("\t {} \t", event.metadata().level());
            let mut visitor = PrintlnVisitor;
            event.record(&mut visitor);
            println!();
        }
    }

    
}

struct PrintlnVisitor;

impl tracing::field::Visit for PrintlnVisitor {
    

    fn record_str(&mut self, _field: &tracing::field::Field, value: &str) {
        print!("\t {} :|:", value)    
    }

    fn record_error(
        &mut self,
        _field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        print!("\t {} :|:", value)
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        println!("  field={} value={:?}", field.name(), value)
    }
}