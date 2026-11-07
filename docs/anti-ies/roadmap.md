---
type: Guide
title: Roadmap IES e anti-IES — primeira capacidade integrada
description: Sequência de PRs e critérios diferenciais para evoluir cliente e servidor no mesmo workspace
---

# IES ↔ anti-IES: evolução conjunta

## Decisão

O IES continua sendo **cliente Web Rust** do SEI; o anti-IES é um
**servidor Rust** experimental no mesmo workspace. O objetivo é ganhar
um ambiente local capaz de exercitar o próprio IES e evoluir, por operação,
até reproduzir o comportamento observável do SEI. O SEI de treinamento e
as [cassettes verificadas](../cassettes.md) são o **oráculo externo**; a
saída do IES não pode ser usada como única fonte para construir a saída
esperada do anti-IES. O anti-IES tampouco serve como substituto do SEI
real ou como prova de equivalência entre instalações.

**Dados e eficiência:** o IES acessa o SEI autorizado, interpreta as respostas e
alimenta o [SQLite companion](sqlite-companion.md) com fatos efetivamente
observados. O anti-IES **somente lê essa projeção** para gerar respostas;
não coleta dados no SEI. Desconhecido não significa ausente.
A [modelagem incremental](modelo-entidades.md) separa identidade,
observação e estado simulado e exige baseline de ingestão/consulta/HTTP
antes de ampliar a cobertura.

Esta trilha complementa o [roadmap operacional #56](https://github.com/franklinbaldo/ies/issues/56),
as [matrizes da CLI](../matriz-derivacao-cli.md) e o [contrato de unidade atual](../../contracts/cli/unidade-atual.md).
Não introduz comandos públicos, não impõe uma segunda fila de capacidade
a cada rotina e **não bloqueia** PRs de PDF, documento, processo e tramitação
que tenham sua própria evidência e gates.

## Primeira capacidade escolhida: unidade.atual

O comando existente **ies unidade atual --json** é leitura do Controle
com **um GET** quando já há sessão válida. A evidência identifica o
marcador **lnkInfraUnidade** e uma página de Controle verificável nas
visões resumida (R) ou detalhada (D); marcador isolado não basta.
O resultado JSON de sucesso tem os campos **unidade_exibida** e
**origem = controle_de_processos**. As recusas de login, tabela ausente,
unidade não identificada e divergência de unidade precisam continuar
detectáveis. Não há POST, paginação, troca de unidade ou escrita.

Fontes: [contrato](../../contracts/cli/unidade-atual.md),
[evidência de unidade](../../contracts/evidencias/unidade-atual.md),
[cassette UnB do Controle](../../cassettes/unb/unidade-processos-paginado-unb--e02dbcfde8c4.md),
[cassette GO do Controle](../../cassettes/go/unidade-painel-controle-go--501a577214db.md),
[ensaios de leitura](../validacao-live/2026-11-02-leitura-controle.md)
e os testes existentes em **src/controle_reconhecido/tests.rs**.
O corpus deve ser inspecionado antes de afirmar que determinada
cassette comprova a forma D.

## PRs previstas até a primeira integração

| Passo | Issue / PR | Entrega e condição para avançar |
| --- | --- | --- |
| 0 — infraestrutura | [#350](https://github.com/franklinbaldo/ies/issues/350) / [PR #351](https://github.com/franklinbaldo/ies/pull/351) | Binário local anti-IES, health, HTTP 501 para ações sem contrato e testes; **PR Draft, CI verde, aguardando revisão/merge**. |
| 0b — governança | [#356](https://github.com/franklinbaldo/ies/issues/356) / PR de planejamento | AGENTS.md, roadmap #56, matriz separada, sequência vinculada e CI OKF; não modifica produção. |
| 1 — oráculo | [#353](https://github.com/franklinbaldo/ies/issues/353) / PR P1 | Cassettes conferidas, contrato R/D, testes positivos e negativos **independentes do servidor**, prova de equivalência definida. |
| 1.2 — modelo e eficiência | [#360](https://github.com/franklinbaldo/ies/issues/360) / PR P1.2 | Tipos de entidade/observação e cobertura, projeção relacional mínima, índices por consulta e baseline reproduzível do SQLite/HTTP. |
| 1.5 — companion | [#358](https://github.com/franklinbaldo/ies/issues/358) / PR P1.5 | **IES** registra no SQLite Controle/unidade/visão e listas observadas; consulta SQL testada, sem novos requests. |
| 2 — produtor | [#354](https://github.com/franklinbaldo/ies/issues/354) / PR P2 | GET do Controle na rota observada; HTML e MIME gerados **a partir de uma projeção SQLite lida em modo read-only**. Estados não observados continuam recusados. |
| 3 — cruzamento | [#355](https://github.com/franklinbaldo/ies/issues/355) / PR P3 | Teste real do circuito **IES → SQLite → anti-IES → IES** em loopback e comparação com oráculo independente. Só este passo pode marcar a capacidade **INTEGRADA**. |

**Dependências:** P1 só inicia sobre main que contém bootstrap e
governança. P1.2 vem após P1; P1.5 depende do **modelo P1.2**,
P2 após P1.5 e P3 após P2. Cada
issue gera uma PR Draft independente, partindo da main atual, e os
critérios do AGENTS.md de revisão/CI/merge por SHA prevalecem.
**Liberação automática da fila:** a 🐘 integra a PR anterior e então
aplica `handoff-chatgpt` **somente** à próxima issue desbloqueada;
P2/P3 ficam sem essa etiqueta até suas dependências estarem na main.
Não reservar uma issue bloqueada por outra PR.

Se qualquer PR ficar WIP, registrar resultado e continuar a própria
issue sem declarar o marco concluído. Não criar branch nova sobre branch
de outro agente nem fazer rebase/force-push.

## Critério verificável de INTEGRADA

1. O IES alimenta um SQLite temporário com uma observação validada e o anti-IES a consulta sem gravar novos fatos.
2. A resposta é **servida por HTTP** pelo processo anti-IES em loopback,
   não apenas entregue como string a um parser unitário.
3. O **caminho real da operação IES** interpreta essa resposta e publica
   o JSON definido no contrato, com unidade correta, origem observada e
   saída 0 (ou erro tipado para os negativos).
4. Os casos R e D, quando fundamentados, mais login disfarçado, tabela
   ausente e unidade divergente são verificados com saídas esperadas
   derivadas do SEI/cassettes, **não** do renderizador anti-IES.
5. A bancada conta **um GET, zero POST**, nenhuma requisição externa,
   nenhum retry, nenhuma escrita e nenhum resíduo em SEI.
6. A bancada nunca relaxa TLS, validade de origem ou isolamento de
   cookies do IES em produção. Um túnel TLS ou ponto de injeção
   **somente de teste**, sem nova flag pública, pode ser usado. O servidor
   HTTP de desenvolvimento permanece em 127.0.0.1.
7. Há benchmark offline de ingestão, SELECT e HTTP, com p50/p95,
   pico de memória e contagem de queries, comparado à baseline da #360;
   medições de velocidade não substituem testes de equivalência.
8. O CI do **head exato** passa com Rustfmt, Clippy, testes do workspace,
   OKF e gates existentes. A issue registra RED → GREEN e quaisquer
   itens live **NOT_RUN** explicitamente.

**Não contar como integração:** apenas um teste da função de roteamento,
um HTML coincidente com fixture, um parser aplicado diretamente a uma
string, CI verde de outro SHA ou resultado gerado pelo mesmo código
usado para compor o esperado. Não confundir esse marco com compatibilidade
completa do SEI ou interoperabilidade de importação pelo SEI oficial.

## Como o anti-IES cresce junto às issues IES

Para cada capacidade **já existente ou nova** na matriz do IES,
a issue/PR deve registrar de forma breve: (a) cenário do SEI de
treinamento e fonte da evidência; (b) contrato de entrada/saída/erro e
efeitos; (c) estado correspondente na
[matriz anti-IES](cobertura.md): não coberta, oráculo pronto, servidor
pronto ou integrada; (d) caso diferencial que falta. Esse registro
pode ser simplesmente **pendente**, sem exigir código do anti-IES na
PR original. Especificamente, uma PR de cliente de alta prioridade
**não é bloqueada** pela falta de emulação: cria-se ou referencia-se
um handoff dedicado, preservando as reservas das rotinas.

Quando houver oráculo e capacidade do cliente, o servidor implementa
**somente** aquela fatia, com estado e erros tipados, sem copiar parser
do cliente, nem transformar HTML observado em modelo universal ou
fabricar comportamento que não apareceu no SEI. Teste diferencial:
uma mesma intenção produz resultados/efeitos normalizados equivalentes,
incluindo ausência, recusa, ordem, paginação e, para escritas,
a leitura posterior. Sem caso no acervo, anotar lacuna e obter
evidência de treinamento de modo autorizado antes de inferir.

Depois de unidade.atual: expandir por proximidade de casos existentes
para **unidade listar-processos → processo NUP → documento DOC**.
Escritas (PDF, documento nativo, tramitação) ficam em trilha posterior
com transições de estado, prévia/confirmar e conferência independente;
não simular sucesso nem despriorizar as issues atuais de produção.

## Responsabilidade e gates

- **Implementadoras:** cada issue mantém seu prefixo; a PR IES descreve
  cobertura anti-IES e aponta handoff quando necessário. O anti-IES pode
  crescer depois, sem duas PRs concorrentes no mesmo arquivo.
- **🦅 Gavião:** verifica que o rótulo INTEGRADA corresponde a um
  teste cruzado real e oráculo independente, sem regressões/segredos.
- **🐘 Elefante:** gerencia a sequência de dependências na #56, só
  libera a issue seguinte depois do merge anterior e registra SHA/CI;
  não interrompe a fila maior para fabricar paridade.
- **🦡 Texugo:** pode simplificar a implementação e reduzir abstrações/
  testes redundantes segundo sua ficha; não transformar uma comparação
  autorreferencial em prova de equivalência.

O estado verificável está em [cobertura](cobertura.md). Atualize-o com
links para a PR e o teste responsável **após** cada marco comprovado.
