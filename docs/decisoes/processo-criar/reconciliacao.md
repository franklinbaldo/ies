---
type: ImplementationDecision
title: Reconciliação e resposta do criador na interface Web
operation: processo.criar
implementation: partial
status: adopted-for-scope
scope: retratos-gerados-verificacao-web-e-recusa-explicita
selected_strategy: leitura-do-controle-e-conferencia-dos-candidatos
alternatives: repetir-post,atribuir-candidato-sem-prova,conferir-retratos-e-identidades
evidence_level: testes-sinteticos-e-replay-publico
live_validation: not-run
---

# Decisão de reconciliação da criação de processo — #245

Continuação da [decisão de criação](../processo-criar.md), sem alterar os
critérios de prévia, seleção do tipo ou submissão única.

## Continuação da reconciliação: retratos de Gerados (#245)

A conciliação histórica **já confirmou por leitura** que a única
confirmação incerta da UnB de treinamento foi aplicada:
NUP `23106.001875/2026-93`, id `51731`
([evidência](../validacao-live/2026-11-05-processo-criar-reconciliacao.md)).
Essa confirmação **não será repetida**. O relato anterior de efeito
inconclusivo se refere à resposta da CLI no momento do POST, não ao
resultado conhecido posteriormente.

Para o mecanismo automático ainda pendente, a primeira fatia escolhe
reutilizar `paginas::juntar` e a tabela tipada de Gerados em vez de
fazer novo parser ou inferir IDs a partir de NUPs.
A fotografia exige cobertura integral de uma página, unidade
coincidente e IDs anteriores estáveis. Uma página paginada é
**inconclusiva**, mesmo com ausência aparente de candidatos.
Um ID que desapareceu ou mudou de NUP também impede conclusão.
Quando há dois candidatos, ambos permanecem candidatos, sem escolher
um por ordem nem inferir autoria da criação. A diferença é uma
**observação**, não ainda prova de aplicação ou de segurança de retry.

Foram considerados (1) comparar apenas a primeira página sem verificar
total, (2) inferir criação pelo maior ID e (3) comparar tabelas tipadas
com cobertura demonstrada. Escolhida a terceira alternativa: mais
restritiva, porém evita falso negativo após uma escrita.
A próxima etapa deve obter Gerados antes/depois, verificar tipo e
especificação dos candidatos por leitura independente e preservar a
causa original do POST sem expor URLs/cookies. Os quatro desfechos
exigidos pela issue continuam pendentes; não liberar novos
`--confirmar` para ensaios de efeito incerto.

### Guarda anterior à escrita, continuação da #245

A observação indireta do Controle mostrou que a tabela Gerados
publica total/faixa, mas pode apresentar só uma página. Antes de enviar
uma escrita cuja resposta talvez precise ser reconciliada, exigir
fotografia íntegra da leitura inicial, reutilizando
`FotografiaDosGerados::da_tela` (0 requisições adicionais).
Alternativas: escrever mesmo com lista parcial (não garante candidatos
posteriores), percorrer automaticamente páginas (mais requests e
muda a tela da sessão), ou recusar de forma tipada. Foi escolhida a
recusa segura; **`--confirmar` não envia POST** quando a cobertura
é insuficiente, mas prévias só de leitura continuam disponíveis.
Os testes novos verificam 3 GET / 0 POST na recusa e na prévia.
A fotografia é preservada no escopo do fluxo, ainda não consumida
após falha: falta a reconciliação Web completa e a verificação independente
de tipo/especificação, sem repetir escritas. Esta guarda restringe
a confirmação e não constitui validação autenticada ao vivo.

### Releitura única depois de falha do POST — 06/11 (#245)

A evidência do ensaio UnB anterior mostrou uma criação real mesmo
quando o POST voltou à CLI com saída 3. Foi rejeitada a alternativa
de repetir o POST, seguir o redirect ou afirmar não aplicação só porque
o NUP não surgiu na resposta. O caminho escolhido reutiliza a
fotografia completa prévia de Gerados e, exclusivamente após falha
de submissão já iniciada, lê uma única vez o Controle para obter
candidatos novos, preservando a causa original da falha e também
a da releitura, quando houver. Contagem sintética: 5 requests
(3 GET, 1 POST criador e 1 GET de conferência).

Esse passo é conservador: mesmo com zero candidatos, não se afirma
que a escrita falhou; outros processos e efeitos de sessão podem
interferir. A saída permanece 3 até leitura verificadora de cada
candidato com ID/NUP, tipo e especificação. O classificador puro
exige essas quatro provas, rejeita parcialidade e mantém ambiguidade
quando vários candidatos correspondem. Ainda falta ligar
o classificador ao transporte e à apresentação dos quatro desfechos.
Nenhuma escrita, leitura ou ensaio live novo foi efetuado nesta fatia.

## Resposta do criador sem identidade (fatia de 06/11/2026)

Adotada uma reconciliação conservadora também para respostas não excepcionais
**inesperadas** após o POST criador: HTML 200 em lugar de redirecionamento, ou
302 com `Location` inválida/ambígua. Sem seguir a `Location`, o fluxo pede
exatamente uma leitura do Controle originalmente observado, compara as
fotografias completas dos Gerados e preserva a causa distinta da resposta e
a falha da releitura, quando houver. Um único POST; nenhum retry. Publicar
candidatos não é inferir efeito. Por isso a saída permanece 3 até conferir
propriedades de todos os candidatos e decidir o resultado; a reconciliação
automática conclusiva segue pendente. Essa alternativa usa a página de
Controle já conhecida, sem tentar inventar o endereço do processo novo.

### Candidatos verificados pela Web (#245)

A próxima fatia usa links **servidos na linha Gerados**, nunca links
inferidos de IDs, e os leitores já disponíveis de tela, árvore e
propriedades. Para cada candidato é necessário comprovar unidade,
ID/NUP, tipo e especificação. Retornar sucesso é permitido apenas para
uma correspondência exata com todos os candidatos conferidos. Sem
propriedades publicadas, falha fechado e preserva a causa do POST.
O novo leitor consome 1 GET da caixa e 3 GET por candidato; não faz
retry nem novo POST. A ausência de candidatos não prova não aplicação.
O ensaio live nesta fatia não foi executado.

### Recusa explícita e concorrência (06/11, #245)

Decisão: validação Infra ou HTTP 4xx após POST encerra com a causa
original, sem GET de Gerados; um candidato idêntico pode ser de outro
agente. Sem resposta e 5xx seguem reconciliação sem retry. Teste RED
adversarial com a conta compartilhada precedeu a correção.


### Classificação de múltiplos candidatos (06/11, #245)

Quando a releitura observa dois processos novos e a conferência da
identidade, da unidade, do tipo e da especificação confirma **ambos**,
nenhum deles prova autoria do POST que falhou. Adotado um desfecho
público distinto, `candidatos_ambiguos_na_criacao`, com saída 3,
NUPs comprovadamente compatíveis e causa do POST preservada.
Alternativas rejeitadas: escolher o primeiro e contar candidatos como
prova de criação; ou esconder a ambiguidade sob o diagnóstico genérico.
Caso haja uma só correspondência, todas as demais candidatas ainda
precisam de leitura independente antes da conclusão. Se não houver
correspondência, não afirmar não aplicação nem permitir retry.

A fatia é suportada por dois cenários de transporte sintético de 11
requisições, cada qual com um único POST. O ensaio ao vivo em segunda
instalação de treinamento permanece NOT_RUN: zero requisições e escritas.
