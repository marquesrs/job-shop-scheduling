package jss


import "core:fmt"
import "core:os"
import "core:time"
import "core:slice"
import "core:mem"
import "core:encoding/csv"
import "core:math"
import "core:math/rand"
import "core:io"

Task :: struct {
	exec_time: int,
}

Machine :: struct {
	tasks: [dynamic]Task,
	makespan: int,
}

machine_create :: proc() -> (m: Machine) {
	m.tasks = make([dynamic]Task)
	return
}

machine_destroy :: proc(m: ^Machine){
	delete(m.tasks)
}

machine_add_task :: proc(m: ^Machine, task: Task){
	append(&m.tasks, task)
	m.makespan += task.exec_time
}

machine_add_task_list :: proc(m: ^Machine, tasks: []Task){
	append(&m.tasks, ..tasks)
	acc := 0
	for t in tasks { acc += t.exec_time }
	m.makespan += acc
}

// machine_makespan :: proc(m: Machine) -> int {
// 	span := 0
// 	for t in m.tasks {
// 		span += int(t)
// 	}
// 	return span
// }

machine_del_task :: proc(m: ^Machine, task_id: int) -> Task {
	t := m.tasks[task_id]
	unordered_remove(&m.tasks, task_id)
	m.makespan -= t.exec_time
	return t
}

machine_highest_task :: proc(m: Machine) -> (max_task: Task, max_idx: int) {
	if !(len(m.tasks) > 0){
		fmt.println(m)
	}

	for task, i in m.tasks {
		if task.exec_time > max_task.exec_time {
			max_task = task
			max_idx = i
		}
	}

	return
}

MachineGroup :: struct {
	machines: []Machine,
}

group_create :: proc(mach_count: int) -> (group: MachineGroup) {
	group.machines = make([]Machine, mach_count)
	for &mach in group.machines {
		mach = machine_create()
	}
	return
}

group_destroy :: proc(mg: ^MachineGroup){
	for &mach in mg.machines {
		machine_destroy(&mach)
	}
	delete(mg.machines)
}

group_most_loaded_machine :: proc(mg: MachineGroup) -> (mach_idx: int) {
	assert(len(mg.machines) > 0, "No machines")

	max_load := 0
	for mach, id in mg.machines {
		span := mach.makespan
		if span > max_load {
			max_load = span
			mach_idx = id
		}
	}
	assert(mg.machines[mach_idx].makespan > 0)
	return
}

group_least_loaded_machine :: proc(mg: MachineGroup) -> (mach_idx: int) {
	assert(len(mg.machines) > 0, "No machines")

	min_load := max(int)
	for mach, id in mg.machines {
		span := mach.makespan
		if span < min_load {
			min_load = span
			mach_idx = id
		}
	}
	return
}

group_transfer_task :: proc(mg: ^MachineGroup, source_id, dest_id: int, task_id: int){
	source := &mg.machines[source_id]
	dest := &mg.machines[dest_id]

	task := source.tasks[task_id]
	machine_del_task(source, task_id)
	machine_add_task(dest, task)
}

// Transfer the highest task from source to destination
group_transfer_highest :: proc(mg: ^MachineGroup, source_id, dest_id: int){
	source := &mg.machines[source_id]
	dest := &mg.machines[dest_id]

	task, task_idx := machine_highest_task(source^)
	machine_del_task(source, task_idx)
	machine_add_task(dest, task)
}

group_makespan :: proc(mg: MachineGroup) -> int {
	start := time.now()
	span := 0
	for mach in mg.machines {
		span = max(span, mach.makespan)
	}
	time_on_group_makespan += time.since(start)
	return span
}

group_select_neighbor_random :: proc(mg: MachineGroup, target: int) -> int {
	assert(len(mg.machines) > 1, "No possible neighbors")
	for {
		idx := int(rand.int31_max(cast(i32)len(mg.machines)))
		if idx != target {
			return idx
		}
	}
}

group_select_neighbor_least_loaded :: proc(mg: MachineGroup, target: int) -> int {
	assert(len(mg.machines) > 1, "No possible neighbors")
	neighbor := group_least_loaded_machine(mg)

	if neighbor == target {
		return (target + 1) % len(mg.machines)
	}
	return neighbor
}

machine_task_count :: proc(m: Machine) -> (count: int, total_load: int) {
	count = len(m.tasks)
	total_load += m.makespan
	return
}

time_on_group_makespan : time.Duration

simmulated_annealing :: proc(mg: ^MachineGroup, initial_temp: f64, alpha: f64) -> ExecInfo {
	current := group_most_loaded_machine(mg^)

	TARGET_TOLERANCE :: 1.005

	target, iter_limit : int
	{
		current_machine := &mg.machines[current]
		task_count, max_load := machine_task_count(current_machine^)

		mean := math.ceil(f64(max_load) / f64(len(current_machine.tasks)))

		target     = 1 + int(TARGET_TOLERANCE * mean * f64(task_count)) / len(mg.machines)
		iter_limit = len(current_machine.tasks)
	}

	return simmulated_annealing_ex(mg, initial_temp, alpha, target, iter_limit)
}

ExecInfo :: struct {
	iteration: int,
	makespan: int,
	time_elapsed: time.Duration,
}

local_best_search :: proc(mg: ^MachineGroup) -> (info: ExecInfo){
	best_makespan := group_makespan(mg^)

	start := time.now()

	best_improvement := min(int)

	for {
		defer info.iteration += 1

		most_loaded_id := group_most_loaded_machine(mg^)
		most_loaded := mg.machines[most_loaded_id]

		max_task, max_task_id := machine_highest_task(most_loaded)

		best_dest_id := most_loaded_id
		for mach, i in mg.machines {
			new_makespan := max(most_loaded.makespan - max_task.exec_time, mach.makespan + max_task.exec_time)
			improvement := best_makespan - new_makespan

			if improvement > best_improvement {
				best_improvement = improvement
				best_dest_id = i
			}
		}

		if best_dest_id != most_loaded_id && best_improvement > 0{
			group_transfer_task(mg, most_loaded_id, best_dest_id, max_task_id)
		}
		else {
			// Could not reach an improvement
			break
		}
	}
	info.time_elapsed = time.since(start)
	info.makespan = group_makespan(mg^)

	return
}

simmulated_annealing_ex :: proc(
	mg: ^MachineGroup,
	initial_temp: f64,
	alpha: f64,
	target: int,
	iter_limit: int,
) -> (info: ExecInfo) {
	previous_makespan := group_makespan(mg^)

	temperature := initial_temp

	iter_count := 0

	start := time.now()
	for {
		temperature = initial_temp * math.pow(alpha, f64(iter_count) * 0.1)
		defer iter_count += 1

		most_loaded_id := group_most_loaded_machine(mg^)
		most_loaded := mg.machines[most_loaded_id]

		max_task, max_task_id := machine_highest_task(most_loaded)

		// First 20% random, remaining 80% more selective
		neighbor_id : int
		if iter_count < (iter_limit / 5) {
			neighbor_id = group_select_neighbor_random(mg^, most_loaded_id)
		}
		else {
			neighbor_id = group_select_neighbor_least_loaded(mg^, most_loaded_id)
		}

		neighbor := mg.machines[neighbor_id]

		new_makespan := max(
			most_loaded.makespan - max_task.exec_time,
			neighbor.makespan + max_task.exec_time
		)

		delta := f64(new_makespan - previous_makespan)

		if delta >= 0 {
		}
		if delta < 0 {
			// Energy was reduced, accept changes
			previous_makespan = new_makespan
			group_transfer_task(mg, most_loaded_id, neighbor_id, max_task_id)
		}
		else {
			// Energy was NOT reduced, randomly decide if answer can be accepted
			p := math.exp(-delta / temperature)
			r := rand.float64_range(0, 1)

			if r < p {
				// Randomly accept, even though it's a worse state
				previous_makespan = new_makespan
				group_transfer_task(mg, most_loaded_id, neighbor_id, max_task_id)
			}
		}

		if new_makespan < target { /* fmt.println("REACHED TARGET"); */ break }
		if iter_count > iter_limit { /* fmt.println("REACHED MAX ITER"); */ break }
	}
	info.time_elapsed = time.since(start)

	info.makespan = previous_makespan;
	info.iteration = iter_count;

	return
}

display_group :: proc(mg: MachineGroup){
	mach_n := len(mg.machines)
	task_n := 0
	for m in mg.machines {
		task_n += len(m.tasks)
	}

	fmt.printfln("--- Makespan: %v | Machines: %v | Tasks: %v ---", group_makespan(mg), mach_n, task_n)
	for mach, id in mg.machines {
		fmt.printf("(S=%d) M{:d} ", mach.makespan, id)
		for task in mach.tasks {
			fmt.printf("%d, ", task)
		}
		fmt.println()
	}
}

generate_random_tasks :: proc(n: int) -> []Task {
	tasks := make([]Task, n, context.temp_allocator)
	for &task in tasks {
		v := rand.int31_max(98) + 1
		task = Task { exec_time = int(v) }
	}
	return tasks
}

demo_prof :: proc(){
	file, error := os.open("out.csv", os.O_WRONLY | os.O_TRUNC)
	if error != nil {
		fmt.panicf("Failed to open output file `out.csv`")
	}
	defer os.close(file)

	stream := os.stream_from_handle(file)

	writer : csv.Writer
	csv.writer_init(&writer, stream)

	csv.write(&writer, {"heuristic", "machines", "tasks", "makespan", "time_nano", "iterations", "param"})

	machine_counts := []int{10, 20, 50}
	exponents := []f64{1.5, 2.0}
	cooling_factors := []f64{0.8, 0.85, 0.9, 0.95, 0.99}

	fmt.println("Running Local Search")
	for m in machine_counts {
		for e in exponents {
			for instance_num in 0..<100 {
				t := int(math.ceil(math.pow(f64(m), e)))
				tasks := generate_random_tasks(t)
				group := group_create(m)
				machine_add_task_list(&group.machines[0], tasks)

				info := local_best_search(&group)
				csv.write(&writer, {
					"LocalSearchBest",
					fmt.tprint(m),
					fmt.tprint(t),
					fmt.tprint(info.makespan),
					fmt.tprint(int(info.time_elapsed / time.Nanosecond)),
					fmt.tprint(info.iteration),
					"",
				})
			}
		}
	}

	fmt.println("Running Simulated Annealing")
	for m in machine_counts {
		for e in exponents {
			for cooling in cooling_factors {
				for instance_num in 0..<100 {
					t := int(math.ceil(math.pow(f64(m), e)))
					tasks := generate_random_tasks(t)
					group := group_create(m)
					machine_add_task_list(&group.machines[0], tasks)

					info := simmulated_annealing(&group, f64(t), cooling)

					csv.write(&writer, {
						"SimulatedAnnealing",
						fmt.tprint(m),
						fmt.tprint(t),
						fmt.tprint(info.makespan),
						fmt.tprint(int(info.time_elapsed / time.Nanosecond)),
						fmt.tprint(info.iteration),
						fmt.tprintf("%.2f", cooling),
					})
				}
			}
		}
	}
}

demo :: proc(){
	machines := 100
	cooling_factors := []f64{0.8, 0.85, 0.9, 0.95, 0.99}

	file, error := os.open("out.csv", os.O_WRONLY | os.O_TRUNC)
	if error != nil {
		fmt.panicf("Failed to open output file `out.csv`")
	}
	defer os.close(file)

	stream := os.stream_from_handle(file)

	writer : csv.Writer
	csv.writer_init(&writer, stream)

	csv.write(&writer, {"heuristic", "machines", "tasks", "makespan", "time_nano", "iterations", "param"})

	MIN_TASK :: 500
	MAX_TASK :: 10_000
	STEP_TASK :: 25
	arena_memory := make([]byte, 512 * mem.Megabyte)

	arena : mem.Arena
	mem.arena_init(&arena, arena_memory)
	context.allocator = mem.arena_allocator(&arena)

	fmt.println("Running Local Search")
	for t := MIN_TASK; t <= MAX_TASK; t += STEP_TASK {
		defer free_all(context.allocator)
		tasks := generate_random_tasks(t)
		group := group_create(machines)
		machine_add_task_list(&group.machines[0], tasks)

		info := local_best_search(&group)
		csv.write(&writer, {
			"LocalSearchBest",
			fmt.tprint(machines),
			fmt.tprint(t),
			fmt.tprint(info.makespan),
			fmt.tprint(int(info.time_elapsed / time.Nanosecond)),
			fmt.tprint(info.iteration),
			"",
		})
	}

	fmt.println("Running Simulated Annealing")
	for cooling in cooling_factors {
		for t := MIN_TASK; t <= MAX_TASK; t += STEP_TASK {
			defer free_all(context.allocator)
			tasks := generate_random_tasks(t)
			group := group_create(machines)
			machine_add_task_list(&group.machines[0], tasks)

			info := simmulated_annealing(&group, f64(t), cooling)

			csv.write(&writer, {
				"SimulatedAnnealing",
				fmt.tprint(machines),
				fmt.tprint(t),
				fmt.tprint(info.makespan),
				fmt.tprint(int(info.time_elapsed / time.Nanosecond)),
				fmt.tprint(info.iteration),
				fmt.tprintf("%.2f", cooling),
			})
		}
	}
}

main :: proc(){
	demo_prof()
	// demo()
}



// main :: proc(){
// 	T :: 2500
// 	M :: 50
//
// 	Simulated_Annealing: {
// 		tasks := generate_random_tasks(T)
// 		group := group_create(M)
// 		machine_add_task_list(&group.machines[0], tasks)
// 		old_makespan := group_makespan(group)
//
// 		info := simmulated_annealing(&group, T, 0.9)
//
// 		fmt.println("-- Simulated Annealing --")
// 		fmt.println("Old Makespan:", old_makespan)
// 		fmt.println("New Makespan:", group_makespan(group))
// 		fmt.printfln("Improvement: %.2f%%", (f64(old_makespan) / f64(group_makespan(group)) - 1.0) * 100)
// 		fmt.println("Took:", info.time_elapsed)
// 		fmt.println("Iterations:", info.iteration)
// 	}
//
// 	Local_Best_Search: {
// 		tasks := generate_random_tasks(T)
// 		group := group_create(M)
// 		machine_add_task_list(&group.machines[0], tasks)
// 		old_makespan := group_makespan(group)
//
// 		info := local_best_search(&group)
//
// 		fmt.println("-- Local Best Search --")
// 		fmt.println("Old Makespan:", old_makespan)
// 		fmt.println("New Makespan:", group_makespan(group))
// 		fmt.printfln("Improvement: %.2f%%", (f64(old_makespan) / f64(group_makespan(group)) - 1.0) * 100)
// 		fmt.println("Took:", info.time_elapsed)
// 		fmt.println("Iterations:", info.iteration)
// 	}
// }
