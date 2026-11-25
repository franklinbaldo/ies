---
type: Instructions
title: Automação e agendamento de rotinas de agentes no Google Antigravity
description: Como agendar agentes do time IES via comandos de chat /schedule, cron Linux headless ou daemon contínuo
---

# Automação e agendamento de rotinas

O time IES opera de forma orquestrada com agentes especializados representados por avatares emoji, cujas regras e prioridades estão descritas no [AGENTS.md](../AGENTS.md) e nas fichas em `team/`.

Existem três maneiras de manter essas rotinas executando de hora em hora:

---

## 1. Na sessão interativa do Antigravity CLI (`/schedule`)

Se você estiver operando no terminal interativo com o Antigravity (`agy`), utilize o comando nativo `/schedule`.

### Grade completa (8 agentes)

Copie e execute no chat do Antigravity:

```text
/schedule "5 * * * *" Você é o 🐘 Elefante. Leia AGENTS.md e team/elefante.md. Verifique PRs abertas com CI oficial completo 7/7 verde no SHA exato, sincronizadas com a main, e integre as fatias aprovadas via merge commit. Mantenha a fila aberta em dia. No Cafezinho permanente #388, mantenha a moral da orquestra elevada.
```

```text
/schedule "10 * * * *" Você é a 🦊 Raposa. Leia AGENTS.md e team/raposa.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance E2E da CLI orientado por cassettes da operação, valide gates locais e abra PR.
```

```text
/schedule "22 * * * *" Você é a 🦉 Coruja. Leia AGENTS.md e team/coruja.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance E2E de reconciliação de processos via cassettes, valide gates e abra PR.
```

```text
/schedule "34 * * * *" Você é a 🐢 Tartaruga. Leia AGENTS.md e team/tartaruga.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance o próximo E2E ancorado em cassettes, valide gates e abra PR.
```

```text
/schedule "40 * * * *" Você é o 🦅 Gavião. Leia AGENTS.md e team/gaviao.md. Faça revisão técnica independente de todas as PRs abertas na esteira, verifique escopo, E2E com respostas originais das cassettes e emita parecer técnico nas PRs.
```

```text
/schedule "46 * * * *" Você é o 🐙 Polvo. Leia AGENTS.md e team/polvo.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance E2E de documentos via cassettes, valide gates e abra PR.
```

```text
/schedule "52 * * * *" Você é o 🦡 Texugo. Leia AGENTS.md e team/texugo.md. Atue no desempate de PRs, na classificação de prioridades 0-100 da fila aberta, desbloqueio de testes e avanço pragmático da capacidade. Concluída a fatia, visite o Cafezinho permanente #388.
```

```text
/schedule "58 * * * *" Você é o 🦫 Castor. Leia AGENTS.md e team/castor.md. Puxe a próxima issue livre de maior prioridade na fila aberta, avance o próximo E2E ancorado em cassettes, garanta consistência e abra PR.
```

Para listar ou cancelar agendamentos ativos na sessão, digite `/tasks`.

---

## 2. Permanente no servidor Linux via cron (`agy -p`) — recomendado

Para manter os agentes trabalhando autonomamente mesmo com o terminal fechado, utilizamos o modo headless (`agy -p`) gerenciado pelo cron do Linux.

### Scripts fornecidos no repositório

- [scripts/rotina_agentes.sh](../scripts/rotina_agentes.sh): Executor protegido por `flock` **por avatar** (uma rodada por agente, sem bloquear outros agentes), com detecção de executável, logs privados (diretório `0700`, arquivos `0600`) em `/tmp/agy-ies/logs` e timeout de 50 minutos.
- [scripts/instalar_cron.sh](../scripts/instalar_cron.sh): Utilitário de inspeção e instalação automática no `crontab`.

### Instalação rápida

Para inspecionar as linhas do cron geradas para o seu diretório:

```bash
./scripts/instalar_cron.sh --print
```

Para aplicar automaticamente no seu crontab:

```bash
./scripts/instalar_cron.sh --apply
```

Para desinstalar:

```bash
./scripts/instalar_cron.sh --uninstall
```

### Grade cron padrão

As entradas instaladas no `crontab -e` têm a seguinte estrutura:

```cron
5  * * * * /caminho/para/ies/scripts/rotina_agentes.sh elefante >/dev/null 2>&1
10 * * * * /caminho/para/ies/scripts/rotina_agentes.sh raposa   >/dev/null 2>&1
22 * * * * /caminho/para/ies/scripts/rotina_agentes.sh coruja   >/dev/null 2>&1
34 * * * * /caminho/para/ies/scripts/rotina_agentes.sh tartaruga >/dev/null 2>&1
40 * * * * /caminho/para/ies/scripts/rotina_agentes.sh gaviao   >/dev/null 2>&1
46 * * * * /caminho/para/ies/scripts/rotina_agentes.sh polvo    >/dev/null 2>&1
52 * * * * /caminho/para/ies/scripts/rotina_agentes.sh texugo   >/dev/null 2>&1
58 * * * * /caminho/para/ies/scripts/rotina_agentes.sh castor   >/dev/null 2>&1
```

---

## 3. Daemon em background contínuo (`scripts/daemon_agentes.sh`)

Em ambientes que não possuem serviço `cron` ativo (como containers, WSL sem systemd ou sessões `tmux`), utilize o daemon em loop:

```bash
nohup ./scripts/daemon_agentes.sh > /tmp/agy-ies-daemon.log 2>&1 &
```

O daemon calcula o próximo minuto agendado, aguarda via `sleep` e dispara as rotinas sem aguardar a conclusão das anteriores. Cada agente tem trava própria, impedindo apenas execuções simultâneas do **mesmo avatar**. Use cron **ou** daemon, nunca os dois ao mesmo tempo.

---

## 4. Monitoramento e Logs

- Os logs ficam em `/tmp/agy-ies/logs/<timestamp>_<avatar>.log` com diretório privado `0700` e arquivo `0600` (incluindo diretorio preexistente).
- Cada avatar tem a própria trava privada em `/tmp/agy-ies/logs/.<avatar>.lock`. Uma rodada que encontrar o mesmo avatar ativo registra que foi pulada; os **outros** avatares continuam executando. `IES_LOCK_FILE` permite travamento global apenas se o operador o configurar explicitamente.
