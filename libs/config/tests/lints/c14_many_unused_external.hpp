// More than 5 unused externals, so the lint collapses them into a single
// summary diagnostic instead of one per class.
class CfgVehicles {
    class alpha;
    class bravo;
    class charlie;
    class delta;
    class echo;
    class foxtrot;
    class golf;
    class used_me;
    class child: used_me {};
};
