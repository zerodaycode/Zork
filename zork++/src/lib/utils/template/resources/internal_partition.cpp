module partitions:internal_partition;

// A module implementation may implement a declaration imported from
// it's relative module interface, but shall not export anything
int just_a_42() {
    return 42;
}