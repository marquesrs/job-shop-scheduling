
type Task = u32;

type MachineId = usize;

pub struct Machine {
    tasks: Vec<Task>,
}

impl Machine {
    pub fn makespan(&self) -> u32 {
        let mut acc = 0u32;
        for task in &self.tasks {
            acc += *task;
        }
        return acc;
    }
    
    pub fn new() -> Self {
		Self {
			tasks: Vec::new(),
		}
	}
	
	pub fn from_tasks(tasks: Vec<Task>) -> Self {
		Self {
			tasks: tasks,
		}
	}
}

pub struct MachineGroup {
    machines: Vec<Machine>,    
}

impl MachineGroup {
    pub fn new(mach_count: usize) -> Self {
        let mut group= MachineGroup {
            machines: Vec::new(),
        };
        for _ in 0..mach_count {
            group.machines.push(Machine::new());
        }
        return group;
    }

    pub fn group_makespan(&self) -> u32 {
        let mut acc = 0u32;
        for mach in &self.machines {
            acc += mach.makespan();
        }
        return acc;
    }
    
    pub fn max_makespan_machine(&self) -> MachineId {
	    let mut max = 0;
	    let mut max_idx = 0;
	    for i in 0..self.machines.len() {
	        if self.machines[i].makespan() > max {
	            max = self.machines[i].makespan();
	            max_idx = i;
	        }
	    }
	    return max_idx;
    }
    
    pub fn neighbor_id(&self, target: MachineId) -> Option<MachineId> {
        let neighbor = target + 1;
        if neighbor > self.machines.len() {
            return None;
        }
        else {
            return Some(neighbor);
        }
    }

    pub fn peek_task(&self, mach_id: MachineId) -> Option<Task> {
        let n = self.machines[mach_id].tasks.len();
        if n == 0 {
            return None;
        }
        return Some(self.machines[mach_id].tasks[n - 1]);
    }
    
    fn pop_task(&mut self, mach_id: MachineId) -> Option<Task> {
        return self.machines[mach_id].tasks.pop();
    }
    
    fn push_task(&mut self, mach_id: MachineId, task: Task){
        self.machines[mach_id].tasks.push(task);
    }
    
    pub fn transfer_task(&mut self, source_id: MachineId, dest_id: MachineId) -> bool {
        match self.pop_task(source_id) {
            Some(e) => {self.push_task(dest_id, e); true}
            None => false
        }
    }
}

pub fn display_group(group: &MachineGroup){
    for mach_id in 0..group.machines.len() {
        print!("M{}: ", mach_id);
        for task in &group.machines[mach_id].tasks {
            print!("{} ", task);   
        }
        println!("\n");
    }
}

pub fn display_info(
    machine: MachineId,
    makespan: u32,
    neighbor_makespan: u32,
    task: u32
) {
	println!(
	    "Machine M{} \nMakespan: {}\nNeigbor makespan: {}\nTask Time: {}\n", 
	    machine+1, 
	    makespan, 
	    neighbor_makespan, 
	    task
	   );
}

fn local_search(mg: &mut MachineGroup) {
    if mg.machines.len() < 2 {
        return;
    }
    
    loop {
        let source_id = mg.max_makespan_machine();
    
        let neighbor_id = match mg.neighbor_id(source_id){
            Some(id) => id,
            None => break,
        };
        
        let task_time = match mg.peek_task(source_id) {
            Some(e) => e,
            None => break,
        };
        
        let source_makespan = mg.machines[source_id].makespan(); 
        
        let dest_makespan = mg.machines[neighbor_id].makespan();
        
        if task_time + dest_makespan <  source_makespan {
            display_info(source_id, source_makespan, dest_makespan, task_time);
            mg.transfer_task(source_id, neighbor_id);
        }
        else {
            break;
        }   
    }
}

pub fn main(){
    let mut group = MachineGroup::new(3);
    let tasks : Vec<Task> = vec![
        6, 1, 4, 5,
    ];
    
    for task in tasks {
        group.push_task(0, task);
    }
    display_group(&group);
    
    local_search(&mut group);
    
    display_group(&group);
}