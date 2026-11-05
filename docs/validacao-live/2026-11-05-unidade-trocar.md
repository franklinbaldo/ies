---
type: SEIObservation
title: Troca de unidade live na UFRJ 4.1.5 e limpeza do resíduo dos ensaios de processo enviar
description: ies unidade trocar PR-P → PR-A → PR-P na UFRJ 4.1.5 de treinamento (conta servidor1), com prévia, verificação pelo Controle novo e requisições contadas no relay CONNECT; em PR-A, concluir e enviar autorizados limparam o resíduo do #173; PASS em todos os passos, 05/11/2026
---

# Live de `unidade trocar` — 05/11/2026

Contrato em [unidade trocar](../../contracts/cli/unidade-trocar.md);
resíduo descrito em [processo enviar, escrita](2026-11-04-processo-enviar-escrita.md).
Autorizado pelo proprietário (#205): a troca PR-P ↔ PR-A, `processo
concluir 99079.900901/2026-02` em PR-A e `processo enviar
99079.901790/2026-43 --para PR-P` em PR-A, só na UFRJ de treinamento.

Binário `ies` do commit `4d2d0bd` (release, copiado para fora do
repositório). Sessão já aberta da UFRJ 4.1.5 (conta pública `servidor1`,
unidade PR-P), copiada para um diretório privado `0700` com arquivos
`0600` apontados pelas variáveis `IES_*_FILE`; nenhum `ies login`. Cada
requisição passou por um relay `CONNECT` local que registra um túnel TLS
por conexão; 32 s ou mais entre comandos; nenhum retry; nenhuma saída 3.
Nenhuma URL, cookie ou `infra_hash` nas saídas (conferido por busca).
Horários em UTC. O container reiniciou antes do live; nenhum comando
live tinha sido enviado antes da reinicialização, e o primeiro comando
foi uma leitura (`r0`).

| Passo | Item | Veredito |
| --- | --- | --- |
| 1 | Prévia `unidade trocar PR-A` | **PASS** |
| 2 | `unidade trocar PR-A --confirmar` | **PASS** |
| 3 | `unidade atual` depois da troca | **PASS** |
| 4a | `processo concluir 99079.900901/2026-02` em PR-A (prévia e `--confirmar`) | **PASS** |
| 4b | `processo enviar 99079.901790/2026-43 --para PR-P` em PR-A (prévia e `--confirmar`) | **PASS** |
| 5 | `unidade trocar PR-P --confirmar` | **PASS** |
| 6 | Leituras: os dois processos abertos só em PR-P | **PASS** |
| — | UnB, GO, PEN (troca live) | **NOT_RUN** (fora do plano) |

## Execuções

| Hora | Rótulo | Comando | Saída | Túneis | `requisicoes` |
| --- | --- | --- | --- | ---: | ---: |
| 00:16:21 | r0 | `ies unidade atual --json` | 0 | 1 | — |
| 00:16:54 | p1 | `ies unidade trocar PR-A --json` | 0 | 2 | 2 |
| 00:17:28 | p2 | `ies unidade trocar PR-A --confirmar --json` | 0 | 5 | 5 |
| 00:18:06 | p3 | `ies unidade atual --json` | 0 | 1 | — |
| 00:18:39 | c1 | `ies processo concluir 99079.900901/2026-02 --json` | 0 | 2 | 2 |
| 00:19:13 | c2 | `… --confirmar --json` | 0 | 5 | 5 |
| 00:19:49 | e1 | `ies processo enviar 99079.901790/2026-43 --para PR-P --json` | 0 | 4 | 4 |
| 00:20:25 | e2 | `… --para PR-P --confirmar --json` | 0 | 6 | 6 |
| 00:21:01 | p5 | `ies unidade trocar PR-P --confirmar --json` | 0 | 5 | 5 |
| 00:21:37 | v1 | `ies processo 99079.900901/2026-02 --json` | 0 | 5 | 5 |
| 00:22:12 | v2 | `ies processo 99079.901790/2026-43 --json` | 0 | 5 | 5 |

Total: 41 túneis, igual à soma das requisições (`unidade atual` não
publica `requisicoes`; cada uma abriu 1 túnel).

## Passos 1 a 3 — a troca para PR-A

r0: `unidade_exibida: PR-P`. p1 (prévia), saída 0: `antes: PR-P`,
`pedida: PR-A`, `id_unidade: 110000938`, as 8 unidades oferecidas
(ADCENTRAL, CAXIAS, DECANIA, FCC, MACAE, PR-A, PR-OGT, PR-P, como na
cassette), 13 nomes de campos com `selInfraUnidades` no fim,
`escopo: sessao_da_conta`, nenhum POST.

p2, saída 0: `depois: PR-A`, `alterado: true`, `links_descartados: 27`,
`requisicoes: 5`. Pelos bytes do relay: Controle (~81 KB), tela de troca
(~47 KB), **POST** (1,8 KB enviados, ~6 KB recebidos: o 302), o GET do
`Location` com aviso (~6 KB: o segundo 302) e o Controle de PR-A (~78 KB).
A sequência é a das 22 trocas do acervo. Depois dela, a URL da sessão
gravada é `procedimento_controlar` de `110000938`, sem `msg`, com
`infra_hash` de **192** hexadecimais; o marcador seguiu na geração 1 com a
URL nova; o arquivo de cookie não mudou.

p3: `unidade_exibida: PR-A`, 1 requisição pela URL de 192 gravada: ela
é reutilizável, e o mesmo cookie ficou na unidade nova.

## Passo 4 — resíduo do #173

c1 (prévia em PR-A, 2 requisições pela entrada por id; o índice não tinha
mais link algum): `unidade: PR-A`, `aberto_em: [PR-A, PR-P]`,
`depois_esperado: [PR-P]`. c2, saída 0: `depois: [PR-P]`, `alterado: true`,
sem reconciliação, 5 requisições.

e1 (prévia): `unidade: PR-A`, `aberto_em: [PR-A]`, destino
`PR-P - Pró-Reitoria de Pessoal` (`110000942`), `manter_aberto: false`.
e2, saída 0: `depois: [PR-P]`, `alterado: true`, 6 requisições.

## Passos 5 e 6 — volta a PR-P

p5, saída 0: `antes: PR-A`, `depois: PR-P`, `id_unidade: 110000942`,
`links_descartados: 16` (os colhidos em PR-A), 5 requisições.

v1 e v2 (`unidade: PR-P`, 5 requisições cada, `resolvido_por:
indice_trabalhar`): `99079.900901/2026-02` aberto só em PR-P (atribuído a
`servidor1`); `99079.901790/2026-43` aberto só em PR-P. O resíduo do #173
está limpo.

## Observações

1. **Cookie.** O IES não lê `Set-Cookie` nesses pedidos; o mesmo cookie
   serviu antes e depois das duas trocas, e a unidade acompanhou a sessão.
   Se o servidor enviou `Set-Cookie`, não foi observado.
2. **Nada da unidade anterior reaproveitado.** Cada troca apagou os links
   da instalação (27 e 16); os comandos seguintes acharam a sessão na
   unidade nova e nenhum falhou por link da unidade anterior.
3. **Sessão original.** Os arquivos de onde a sessão foi copiada ainda
   guardam a URL de login de PR-P (64 hexadecimais); se ela ainda abre o
   Controle com a sessão de volta em PR-P **não foi verificado**.
4. Nenhum defeito encontrado.
