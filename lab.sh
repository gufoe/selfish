#!/bin/bash
USER=giacomo.rizzi3
MASTER_USER=giacomo.rizzi3
MASTER=130.136.4.193
CMD=$1
NAME=$2

other_commands() {
    echo "SIGINT caught, killing..."
    echo "bye"
    pkill -P $$
    exit 1
}

trap 'other_commands' SIGINT


start() {
  cargo build --release || exit 1
  # rsync --progress -r target/release/selfish2 $MASTER_USER@$MASTER:/public/selfish/
  rsync --progress -v target/release/selfish2 $MASTER_USER@$MASTER:/public/selfish/ &
  rsync --progress -v config.yaml $MASTER_USER@$MASTER:/public/selfish/$NAME.yaml &
  rsync --progress -v -r data $MASTER_USER@$MASTER:/public/selfish/ &
  bg=`jobs -p`
  for job in $bg; do
    echo "Waiting $job..."
    wait $job || exit 1
  done
  echo "Sync completed."
  echo

  ssh "$MASTER_USER@$MASTER" "killall selfish2; cd /public/selfish/ && screen -L master.log -dm bash -c 'RUST_BACKTRACE=1 ./selfish2 -s 8080 -t 1 -w /tmp/$NAME.json --config $NAME.yaml >> /tmp/$NAME.csv'"
  echo synced server
  sleep 1
  echo starting...
  for HOST in $(cat ../unibo.list);
  do
    echo executing command on $HOST
    # ssh -oStrictHostKeyChecking=no -oPasswordAuthentication=no "$USER@$HOST" "killall selfish2; sleep 2; [[ \$(who) ]] && who || cd /public/selfish/ && screen -L worker-$HOST.log -dm bash -c './selfish2 -c $MASTER:8080 -t 2'" &
    ssh -oStrictHostKeyChecking=no -oPasswordAuthentication=no "$USER@$HOST" "killall selfish2; cd /public/selfish/ && screen -L worker-$HOST.log -dm bash -c 'RUST_BACKTRACE=1 ./selfish2 -c $MASTER:8080 -t 1'" &
  done

  sleep 5
}


stop() {
  ssh $MASTER_USER@$MASTER killall -9 selfish2
  for HOST in $(cat ../unibo.list);
  do
    echo executing command on $HOST
    ssh "$USER@$HOST" "killall -9 selfish2" &
  done
  sleep 5
}

pid() {
  ssh $MASTER_USER@$MASTER pidof selfish2
  for HOST in $(cat ../unibo.list);
  do
    echo executing command on $HOST
    ssh "$USER@$HOST" "pidof selfish2"
  done
  sleep 5
  pkill -P $$
}

case "$CMD" in
  "start")
    start
    ;;
  "stop")
    stop
    ;;
  "pid")
    pid
    ;;
  *)
    echo "You have failed to specify what to do correctly."
    exit 1
    ;;
esac
