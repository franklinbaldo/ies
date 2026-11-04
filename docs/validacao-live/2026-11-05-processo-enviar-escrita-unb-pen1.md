---
type: SEIObservation
title: Escrita live de processo enviar na UnB 4.0.9 e no PEN1 5.0.4
description: Escrita autorizada de ies processo enviar na UnB 4.0.9 (Unidade1 → Unidade2) e no PEN1 5.0.4 (TESTE → TESTE_1_1) de treinamento, com e sem manter aberto, verificada por leitura independente e com as requisições contadas no relay CONNECT local (PASS nos oito passos); registra o resíduo a limpar, 04/11/2026
---

# Escrita live de `processo enviar` na UnB e no PEN1 — 04/11/2026

Contrato em [processo enviar](../../contracts/cli/processo-enviar.md);
método e formato do [relatório da UFRJ](2026-11-04-processo-enviar-escrita.md).
Escrita autorizada pelo proprietário em 04/11/2026 para `processo enviar`,
só em instalação de treinamento. Fecha o critério "Live em UnB e PEN1" da
issue #173.

Binário `ies` do commit `3bf048e` (`main`, copiado para fora do
repositório, sem novo build). Sessões já abertas da UnB 4.0.9 (conta
pública `aluno1`, unidade da sessão `Unidade1`) e do PEN1 5.0.4 (conta
pública `orgao1`, unidade `TESTE`), com o índice de cada sessão; sessão,
senha, cookie e URL do Controle em arquivos privados `0600` apontados
pelas variáveis `IES_*_FILE`. Nenhum `ies login` (nenhuma sessão tinha
vencido). Cada comando passou por um relay `CONNECT` local que registrou
um túnel TLS por conexão; 32 s ou mais entre comandos na UnB e 62 s ou
mais no PEN1 (zona Cloudflare compartilhada); nenhum retry. JSON de cada
comando em arquivo privado fora do repositório; nenhuma URL, cookie ou
`infra_hash` nas saídas (conferido por busca). Horários em UTC; o
andamento do SEI mostra a hora local (UTC−3).

| Instalação | Passo | Item | Veredito |
| --- | --- | --- | --- |
| UnB 4.0.9 | 1 | Destino `Unidade2` (a conta tem `Unidade1` e `Unidade2`) | **PASS** |
| UnB 4.0.9 | 2 | Processos de ensaio criados pelo Todos em `Unidade1` | **PASS** |
| UnB 4.0.9 | 3a | `--manter-aberto --confirmar` em `23106.001868/2026-91` | **PASS** |
| UnB 4.0.9 | 3b | Leitura independente de 3a | **PASS** |
| UnB 4.0.9 | 3c | `--confirmar` **sem** `--manter-aberto` em `23106.001869/2026-36` | **PASS** |
| UnB 4.0.9 | 3d | Leitura independente de 3c | **PASS** |
| PEN1 5.0.4 | 1 | Destino `TESTE_1_1` (oferecido pelo autocomplete; a conta só tem `TESTE`) | **PASS** |
| PEN1 5.0.4 | 2 | Processos de ensaio criados pelo Todos em `TESTE` | **PASS** |
| PEN1 5.0.4 | 3a | `--manter-aberto --confirmar` em `043484/2026-29` | **PASS** |
| PEN1 5.0.4 | 3b | Leitura independente de 3a | **PASS** |
| PEN1 5.0.4 | 3c | `--confirmar` **sem** `--manter-aberto` em `043485/2026-73` | **PASS** |
| PEN1 5.0.4 | 3d | Leitura independente de 3c | **PASS** |
| — | 4 | Saída 3 (efeito incerto) | não ocorreu |

## Escolha do destino (passo 1)

**UnB.** A conta `aluno1` tem permissão em `Unidade1` e `Unidade2`
(tela `infra_trocar_unidade` gravada pelo Todos, `unidade-troca-e-volta-unb`:
"Lista de Unidades com Permissão (2 registros)"), e o envio gravado
(`processo-enviar-retorno-unb`) foi `Unidade1` → `Unidade2`. Destino
`Unidade2`: o proprietário pode receber e limpar com a mesma conta.

**PEN1.** A conta `orgao1` só tem `TESTE` ("Lista de Unidades com
Permissão (1 registro)", `marcador-listar-estados-pen1`). Não há destino
que a conta tenha; o autocomplete gravado oferece `TESTE_1_1 - Unidade de
Testes 1.1` e `TESTE_1_2`, e os envios gravados do Todos usam
`TESTE_1_1`. Destino `TESTE_1_1`, aceito pelo plano (o destino não
precisa ser da conta); o resíduo nele não é limpável com `orgao1`.

## Execuções

| Hora | Inst. | Rótulo | Comando | Saída | Túneis | `requisicoes` |
| --- | --- | --- | --- | --- | ---: | ---: |
| 23:40:30 | UnB | t1 | `todos processo criar --tipo "Administração Geral: Acordos" --acesso publico --descricao "simulação treinamento - ensaio IES processo enviar (1, manter aberto)" --unidade Unidade1 --json` | 0 | 1 | — |
| 23:41:07 | UnB | t2 | o mesmo, com `--confirmar` | 0 | 1 | — |
| 23:41:43 | UnB | t3 | o mesmo com `(2, sem manter aberto)` e `--confirmar` | 0 | 1 | — |
| 23:42:19 | UnB | a1 | `ies processo enviar 23106.001868/2026-91 --para Unidade2 --manter-aberto --json` | 0 | 5 | 5 |
| 23:42:56 | UnB | a2 | `… --para Unidade2 --manter-aberto --confirmar --json` | 0 | 6 | 6 |
| 23:43:34 | UnB | b1 | `ies processo 23106.001868/2026-91 --json` | 0 | 4 | 4 |
| 23:44:10 | UnB | c1 | `ies processo enviar 23106.001869/2026-36 --para Unidade2 --json` | 0 | 5 | 5 |
| 23:44:46 | UnB | c2 | `… --para Unidade2 --confirmar --json` | 0 | 6 | 6 |
| 23:45:24 | UnB | d1 | `ies processo 23106.001869/2026-36 --json` | 0 | 3 | 3 |
| 23:45:35 | PEN1 | t1 | `todos processo criar --tipo "Gestão da Informação: Normatização Interna" --acesso publico --descricao "simulação treinamento - ensaio IES processo enviar (1, manter aberto)" --unidade TESTE --json` | 0 | 1 | — |
| 23:46:42 | PEN1 | t2 | o mesmo, com `--confirmar` | 0 | 1 | — |
| 23:47:48 | PEN1 | t3 | o mesmo com `(2, sem manter aberto)` e `--confirmar` | 0 | 1 | — |
| 23:48:54 | PEN1 | a1 | `ies processo enviar 043484/2026-29 --para TESTE_1_1 --manter-aberto --json` | 0 | 5 | 5 |
| 23:50:01 | PEN1 | a2 | `… --para TESTE_1_1 --manter-aberto --confirmar --json` | 0 | 6 | 6 |
| 23:51:08 | PEN1 | b1 | `ies processo 043484/2026-29 --json` | 0 | 4 | 4 |
| 23:52:14 | PEN1 | c1 | `ies processo enviar 043485/2026-73 --para TESTE_1_1 --json` | 0 | 5 | 5 |
| 23:53:21 | PEN1 | c2 | `… --para TESTE_1_1 --confirmar --json` | 0 | 6 | 6 |
| 23:54:29 | PEN1 | d1 | `ies processo 043485/2026-73 --json` | 0 | 3 | 3 |

Os túneis conferem com `requisicoes` em todos os comandos do IES. O Todos
reusa a conexão (keep-alive): um túnel por comando (UnB: ~6,0 KB
enviados, 105 KB/72 KB/72 KB recebidos; PEN1: ~8,6–9,5 KB enviados, 81–86
KB recebidos); o número de requisições HTTP do Todos **não foi contado**.
O Todos rodou com `HOME` temporário próprio fora do repositório, senha
lida do arquivo privado (nunca em argumento), `SEI_UNIDADE` e `--unidade`
explícitos, e `HTTPS_PROXY`/`https_proxy` apontando para o relay. A
prévia do segundo processo não foi repetida: o plano era o mesmo do t1.

## Passo 2 — processos de ensaio (PASS)

| Inst. | NUP | `id` | Especificação | Saída do Todos |
| --- | --- | --- | --- | --- |
| UnB | `23106.001868/2026-91` | `51708` | "simulação treinamento - ensaio IES processo enviar (1, manter aberto)" | `executado: true`, `verificado: true` |
| UnB | `23106.001869/2026-36` | `51709` | "… (2, sem manter aberto)" | idem |
| PEN1 | `043484/2026-29` | `16864` | "… (1, manter aberto)" | idem |
| PEN1 | `043485/2026-73` | `16865` | "… (2, sem manter aberto)" | idem |

As prévias a1 e c1 do IES confirmaram cada processo aberto só na unidade
da sessão (`aberto_em: [Unidade1]` / `[TESTE]`), e o andamento "Processo
público gerado" saiu na unidade certa. No PEN1 o tipo gera NUP sem o
prefixo de órgão (`043484/2026-29`); o IES o trata como opaco e o aceitou.

## UnB 4.0.9

**3a (PASS).** a1 (prévia, abertura a frio pela linha do Controle): 5
requisições, `aberto_em: [Unidade1]`, `destino: Unidade2 - Unidade2`
(`id_unidade` `110000847`), `depois_esperado: [Unidade1, Unidade2]`. a2:

```json
{"nup": "23106.001868/2026-91", "id_procedimento": "51708", "unidade": "Unidade1",
 "origem": "web", "confirmado": true, "alterado": true, "manter_aberto": true,
 "destino": {"sigla": "Unidade2", "descricao": "Unidade2 - Unidade2", "id_unidade": "110000847"},
 "aberto_em": ["Unidade1"], "depois_esperado": ["Unidade1", "Unidade2"],
 "depois": ["Unidade1", "Unidade2"], "reconciliado_por_leitura": false, "requisicoes": 6}
```

Pelos bytes do relay: tela do processo (~36 KB), árvore (~48 KB), tela
"Enviar Processo" (~26 KB), consulta (~7,1 KB), **POST do envio** (1,7 KB
enviados, ~7,2 KB recebidos; resposta esperada, sem `falha_do_envio`) e a
árvore da verificação (~48 KB): 4 GET + 2 POST.

**3b (PASS).** b1, 4 requisições (`resolvido_por: indice_link`):
`aberto_em` = `Unidade1` e `Unidade2` (ambos `atribuido_a: null`);
andamentos 2 de 2, o mais recente "Processo remetido pela unidade
Unidade1", unidade `Unidade2`, `aluno1`, 04/11/2026 20:43.

**3c (PASS).** c1: 5 requisições, `depois_esperado: [Unidade2]`,
`manter_aberto: false`. c2: `alterado: true`, `aberto_em: [Unidade1]`,
`depois: [Unidade2]`, `reconciliado_por_leitura: false`,
`requisicoes: 6` (POST de 1,7 KB, ~7,2 KB de resposta).

**3d (PASS).** d1, 3 requisições: `aberto_em` = só `Unidade2`;
andamentos "Processo remetido pela unidade Unidade1" (`Unidade2`,
`aluno1`, 20:44) e "Processo público gerado" (`Unidade1`, 20:41);
`marcadores: null` com `tela_de_marcadores_nao_publicada_na_barra`, como
na UFRJ.

## PEN1 5.0.4

**3a (PASS).** a1: 5 requisições, `aberto_em: [TESTE]`,
`destino: TESTE_1_1 - Unidade de Testes 1.1` (`id_unidade` `110000002`),
`depois_esperado: [TESTE, TESTE_1_1]`. a2:

```json
{"nup": "043484/2026-29", "id_procedimento": "16864", "unidade": "TESTE",
 "origem": "web", "confirmado": true, "alterado": true, "manter_aberto": true,
 "destino": {"sigla": "TESTE_1_1", "descricao": "TESTE_1_1 - Unidade de Testes 1.1", "id_unidade": "110000002"},
 "aberto_em": ["TESTE"], "depois_esperado": ["TESTE", "TESTE_1_1"],
 "depois": ["TESTE", "TESTE_1_1"], "reconciliado_por_leitura": false, "requisicoes": 6}
```

Bytes: tela do processo (~44 KB), árvore (~28 KB), tela "Enviar
Processo" (~31 KB), consulta (~4,1 KB), **POST do envio** (2,1 KB
enviados, ~4,2 KB recebidos; resposta esperada) e a árvore da
verificação (~28 KB): 4 GET + 2 POST. Nenhum 403 do Cloudflare.

**3b (PASS).** b1, 4 requisições: `aberto_em` = `TESTE` ("Unidade de
Teste 1") e `TESTE_1_1` ("Unidade de Testes 1.1"); o mais recente
"Processo remetido pela unidade TESTE", unidade `TESTE_1_1`, `ORGAO1`,
04/11/2026 20:50.

**3c (PASS).** c1: 5 requisições, `depois_esperado: [TESTE_1_1]`. c2:
`alterado: true`, `aberto_em: [TESTE]`, `depois: [TESTE_1_1]`,
`reconciliado_por_leitura: false`, `requisicoes: 6`.

**3d (PASS).** d1, 3 requisições: `aberto_em` = só `TESTE_1_1`;
"Processo remetido pela unidade TESTE" (`TESTE_1_1`, `ORGAO1`, 20:53) e
"Processo público gerado" (`TESTE`, 20:47); `marcadores: null` com o
mesmo motivo.

## Observações

1. **Padrão da caixa.** Nas duas instalações, como na UFRJ: com
   `--manter-aberto` o processo fica no destino e na origem; com a caixa
   enviada como servida (desmarcada), sai da origem. Vale para 4.0.9
   (`rdoPrazo`/`txtPrazo`/`txtDias`) e 5.0.4 (retorno e reabertura
   programados).
2. **Contagem.** Prévia a frio 5 (4 GET + 1 POST) e `--confirmar` quente
   6 (4 GET + 2 POST), exatamente o orçamento do contrato, nas duas.
3. **Ordem de `depois`.** Aqui `depois` saiu na mesma ordem de
   `depois_esperado` (a árvore lista a origem primeiro); na UFRJ, não. A
   comparação é por conjunto.
4. **Propriedades.** Nas quatro leituras `propriedades.fonte` foi
   `procedimento_alterar`, também com o processo fechado na unidade da
   sessão (d1); na UFRJ, d1 veio de `procedimento_consultar`. É dado da
   instalação, não defeito; não investigado.
5. **Nenhum defeito de código encontrado** neste ensaio.

## Resíduo para o proprietário limpar no navegador

O IES ainda não troca de unidade; nada foi devolvido nem concluído.

| Inst. | Processo | Estado agora | Limpeza sugerida |
| --- | --- | --- | --- |
| UnB | `23106.001868/2026-91` | aberto em **Unidade1** e **Unidade2** | trocar para Unidade2 e "Concluir"; depois concluir em Unidade1 |
| UnB | `23106.001869/2026-36` | aberto só em **Unidade2** | trocar para Unidade2 e "Concluir" (ou enviar a Unidade1) |
| PEN1 | `043484/2026-29` | aberto em **TESTE** e **TESTE_1_1** | "Concluir" em TESTE; a cópia em TESTE_1_1 **não é limpável** com `orgao1` |
| PEN1 | `043485/2026-73` | aberto só em **TESTE_1_1** | **não limpável** com `orgao1` (precisa de conta com TESTE_1_1) |

O SEI não apaga processo: os quatro continuam existindo, com as
especificações acima, e o andamento "Processo remetido pela unidade …"
fica no histórico.

## Não verificado

- O número de requisições HTTP do Todos (um túnel keep-alive por comando).
- As permissões de unidade das contas vieram de gravações do Todos, não
  foram relidas hoje.
- A limpeza e o estado final depois dela.
- Vários destinos, grupos, retorno programado e GO (escrita).
