---
type: SEIObservation
title: Escrita live de processo enviar na UFRJ 4.1.5
description: Primeira escrita autorizada de ies processo enviar na UFRJ 4.1.5 de treinamento, com e sem manter aberto, verificada por leitura independente e com as requisições contadas no relay CONNECT local (PASS nos quatro passos); mede o padrão da caixa "Manter processo aberto" e registra o resíduo a limpar, 04/11/2026
---

# Escrita live de `processo enviar` — 04/11/2026

Contrato em [processo enviar](../../contracts/cli/processo-enviar.md);
plano e prévia em [ensaio de processo enviar](2026-11-04-processo-enviar.md).
Escrita autorizada pelo proprietário em 04/11/2026 para `processo enviar`
(e `documento assinar`), só em instalação de treinamento e só neste plano.

Binário `ies` do commit `1a4bd85` (copiado para fora do repositório, sem
novo build). Sessão já aberta da UFRJ 4.1.5 de treinamento (conta pública
`servidor1`, unidade PR-P), com o índice da sessão; sessão, senha, cookie
e URL do Controle em arquivos privados `0600` apontados pelas variáveis
`IES_*_FILE`, como nos ensaios anteriores. Nenhum `ies login` (a sessão não
tinha vencido). Cada comando passou por um relay `CONNECT` local que
registrou um túnel TLS por conexão; 31 s ou mais entre comandos; nenhum
retry. JSON de cada comando em arquivo privado fora do repositório;
nenhuma URL, cookie ou `infra_hash` nas saídas (conferido por busca).
Horários em UTC; o andamento do SEI mostra a hora local (UTC−3).

| Passo | Item | Veredito |
| --- | --- | --- |
| 1 | `--manter-aberto --confirmar` em `99079.900901/2026-02` | **PASS** |
| 2 | Leitura independente do processo do passo 1 | **PASS** |
| 3 | `--confirmar` **sem** `--manter-aberto` em `99079.901790/2026-43` | **PASS** |
| 4 | Leitura independente do processo do passo 3 | **PASS** |
| — | Saída 3 (efeito incerto) | não ocorreu |
| — | UnB 4.0.9, PEN1, GO (escrita) | **NOT_RUN** (fora deste plano) |

## Execuções

| Hora | Rótulo | Comando | Saída | Túneis | `requisicoes` |
| --- | --- | --- | --- | ---: | ---: |
| 22:45:45 | a1 | `ies processo enviar 99079.900901/2026-02 --para PR-A --manter-aberto --json` | 0 | 4 | 4 |
| 22:46:20 | a2 | `… --para PR-A --manter-aberto --confirmar --json` | 0 | 6 | 6 |
| 22:46:57 | b1 | `ies processo 99079.900901/2026-02 --json` | 0 | 4 | 4 |
| 22:48:37 | t1 | `todos processo criar --tipo "Pessoal: Avaliação de Estágio Probatório - TAES Nomeados a partir de 07/02/2025" --acesso publico --descricao "simulação treinamento - ensaio IES processo enviar" --json` | 0 | 1 | — |
| 22:49:13 | t2 | o mesmo, com `--confirmar` | 0 | 1 | — |
| 22:49:48 | c1 | `ies processo enviar 99079.901790/2026-43 --para PR-A --json` | 0 | 5 | 5 |
| 22:50:24 | c2 | `… --para PR-A --confirmar --json` | 0 | 6 | 6 |
| 22:51:00 | d1 | `ies processo 99079.901790/2026-43 --json` | 0 | 3 | 3 |

Os túneis contados conferem com `requisicoes` em todos os comandos do
IES (cada requisição do IES abre um túnel). O Todos reusa a conexão
(keep-alive): um túnel por comando, com cerca de 7,0 KB enviados e 76 KB
(t1) e 68 KB (t2) recebidos; o relay não separa as requisições HTTP dentro
do túnel, então o número de requisições do Todos **não foi contado**.

## Passo 1 — enviar mantendo aberto (PASS)

`99079.900901/2026-02` (o mesmo do ensaio de concluir e da prévia) ainda
estava aberto só em PR-P: a prévia a1 publicou `unidade: PR-P`,
`aberto_em: [PR-P]`, `destino: PR-A - Pró-Reitoria de Acesso`
(`id_unidade` `110000938`), `depois_esperado: [PR-P, PR-A]`,
`manter_aberto: true`, `confirmado: false`.

a2 (`--confirmar`), saída 0:

```json
{"nup": "99079.900901/2026-02", "id_procedimento": "3689", "unidade": "PR-P",
 "origem": "web", "confirmado": true, "alterado": true, "manter_aberto": true,
 "destino": {"sigla": "PR-A", "descricao": "PR-A - Pró-Reitoria de Acesso", "id_unidade": "110000938"},
 "aberto_em": ["PR-P"], "depois_esperado": ["PR-P", "PR-A"], "depois": ["PR-A", "PR-P"],
 "reconciliado_por_leitura": false, "requisicoes": 6}
```

Sequência pelos bytes do relay: tela do processo pelo link guardado
(~42 KB), árvore (~29 KB), tela "Enviar Processo" (~30 KB), consulta do
autocomplete (~5,9 KB), **POST do envio** (1,8 KB enviados, ~5,9 KB
recebidos; resposta esperada, pois `falha_do_envio` ficou ausente e
`reconciliado_por_leitura: false`) e a árvore da verificação (~29 KB).
São os 6 previstos com a abertura quente (4 GET + 2 POST).

## Passo 2 — leitura independente (PASS)

b1, saída 0, `requisicoes: 4` (`resolvido_por: indice_link`):
`aberto_em` = PR-A (`atribuido_a: null`) e PR-P (atribuído a
`servidor1`). Andamentos (4 de 4 declarados), o mais recente:
"Processo remetido pela unidade PR-P", unidade PR-A, `servidor1`,
04/11/2026 19:46 (hora local). Confere com o `depois` do passo 1.

## Passo 3 — enviar sem manter aberto (PASS)

**Processo.** Nenhum processo conhecido do acervo e dos relatórios estava
aberto só em PR-P e era do próprio ensaio: o `99079.901789/2026-19`, visto
na varredura, foi gerado por terceiros e tem documento editado. Para não
tirar de PR-P um processo alheio, o agente criou um pelo Todos
(`/home/user/todos`, `uv run todos`), só com `processo criar`, na UFRJ e
com `SEI_UNIDADE=PR-P`: t1 (prévia) e t2 (`--confirmar`), saída 0,
`executado: true`, `verificado: true`, **`numero: 99079.901790/2026-43`**,
`id: 5135`. A saída do Todos não registrou troca de unidade; a prévia c1
confirmou o processo aberto só em PR-P. O Todos fez o próprio login
(sessão em arquivo `0600` num `HOME` temporário fora do repositório,
senha lida do arquivo privado, nunca em argumento).

c1 (prévia, abertura a frio pela linha do Controle), saída 0,
`requisicoes: 5`: `aberto_em: [PR-P]`, `depois_esperado: [PR-A]`,
`manter_aberto: false`.

c2 (`--confirmar`, sem `--manter-aberto`), saída 0:

```json
{"nup": "99079.901790/2026-43", "id_procedimento": "5135", "unidade": "PR-P",
 "origem": "web", "confirmado": true, "alterado": true, "manter_aberto": false,
 "destino": {"sigla": "PR-A", "descricao": "PR-A - Pró-Reitoria de Acesso", "id_unidade": "110000938"},
 "aberto_em": ["PR-P"], "depois_esperado": ["PR-A"], "depois": ["PR-A"],
 "reconciliado_por_leitura": false, "requisicoes": 6}
```

**Padrão medido:** com a caixa "Manter processo aberto na unidade atual"
enviada como servida (desmarcada), o processo **sai** da unidade de
origem: `aberto_em` depois = `[PR-A]`, sem PR-P. A verificação do IES
concorda com a leitura independente do passo 4.

## Passo 4 — leitura independente (PASS)

d1, saída 0, `requisicoes: 3` (`resolvido_por: indice_link`):
`aberto_em` = só PR-A (`atribuido_a: null`). Andamentos (2 de 2):
"Processo remetido pela unidade PR-P" (unidade PR-A, `servidor1`,
04/11/2026 19:50) e "Processo público gerado" (PR-P, 19:49).

Fato novo da leitura, fora da unidade: com o processo fechado em PR-P, a
barra não publica a tela de marcadores (`marcadores: null`,
`marcadores_nao_lidos_motivo: tela_de_marcadores_nao_publicada_na_barra`)
e as propriedades vêm de `procedimento_consultar` (no passo 2, aberto na
unidade, de `procedimento_alterar`); por isso 3 requisições em vez de 4.
O IES publica a ausência como `null` com o motivo, como o contrato
manda; não é defeito.

## Observações

1. **Ordem de `depois`.** No passo 1, `depois` saiu `[PR-A, PR-P]` (a
   ordem da árvore) e `depois_esperado` `[PR-P, PR-A]`; a verificação os
   comparou como conjunto (saída 0, `alterado: true`). Sem defeito: só a
   ordem de apresentação difere.
2. **Contagem.** A frio, a prévia gastou 5 requisições (4 GET + 1 POST),
   como no contrato; quente, cada `--confirmar` gastou 6.
3. **Nenhum defeito de código encontrado** neste ensaio.

## Resíduo para o proprietário limpar no navegador

O IES ainda não troca de unidade; nada foi devolvido nem concluído por
outra ferramenta.

**Limpo em 05/11/2026** pelo IES (`unidade trocar`, #205): em PR-A,
`processo concluir 99079.900901/2026-02` e `processo enviar
99079.901790/2026-43 --para PR-P`; de volta a PR-P, os dois estão abertos
só em PR-P ([relatório](2026-11-05-unidade-trocar.md)).

| Processo | Estado agora | Limpeza sugerida |
| --- | --- | --- |
| `99079.900901/2026-02` | aberto em **PR-A** e em **PR-P** | trocar para PR-A e "Concluir" lá (ou devolver a PR-P) |
| `99079.901790/2026-43` (criado neste ensaio pelo Todos) | aberto só em **PR-A** | trocar para PR-A e "Enviar Processo" a PR-P (ou concluir em PR-A) |

O SEI não apaga processo: `99079.901790/2026-43` continua existindo
(especificação "simulação treinamento - ensaio IES processo enviar"). Os
andamentos "Processo remetido pela unidade PR-P" ficam no histórico dos
dois.

## Não verificado

- O número de requisições HTTP do Todos (um túnel keep-alive por comando).
- Se o login do Todos trocou de unidade (a saída não o diz; o processo
  nasceu em PR-P).
- A volta (PR-A → PR-P) e o estado final só em PR-P: dependem da limpeza
  no navegador.
- Escrita em outras instalações, vários destinos, grupos e retorno
  programado.
