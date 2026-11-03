# Real genetics and game breeding: quick design study

## Scope and evidence

A short source-based study to inform [Define a deep but readable genetic model](../issues/02-readable-genetics.md), not a chosen design. Biology sources are NIH/NHGRI explanations; game sources are developer-authored descriptions and the Axie whitepaper. No games were runtime-tested. The Axie support guide returned HTTP 403; its whitepaper was fetched instead and may describe historical mechanics rather than the current live game. No background-agent facility was available, so this study was conducted directly.

User constraints: gene capacity is separately inherited; genetics constructs a born combat kit; synergy matters across stats, abilities, passives, and teammates; inherited material can remain unexpressed. Previous token/slot recommendations were not accepted. Forty letter-digit tokens remain an illustration, not a commitment.

## What real genetics offers

### 1. Genes are instructions, not miniature finished traits

Genes commonly encode proteins or functional RNA, whose interactions contribute to observable traits. The genome is not a catalog where each entry directly names one ability. Distinguish genotype (genetic constitution) from phenotype (observable characteristics). [NHGRI: Gene](https://www.genome.gov/genetics-glossary/Gene)

**Design inspiration:** abstract variants can influence underlying functional tendencies; several tendencies can jointly produce combat properties. This is an analogy, not a proposal to simulate proteins.

### 2. Alleles are alternative versions, not quality levels

In the ordinary diploid/autosomal model, an individual carries two alleles at a location, one inherited from each parent. An allele is a sequence variant, not inherently a higher or lower quality tier. This model is not universal across all organisms or genomic regions. [NHGRI: Allele](https://www.genome.gov/genetics-glossary/Allele)

**Design inspiration:** a creature can carry variants that do not visibly dominate its current phenotype but matter in breeding. Whether to use paired alleles is still a design decision.

### 3. Dominance is not strength

Dominance describes how alleles relate to an observed trait. In a simple dominant/recessive case, one allele's effect can mask the other's. It does not mean the dominant allele is fitter or more powerful. [NHGRI: Dominant Traits and Alleles](https://www.genome.gov/genetics-glossary/Dominant-Traits-and-Alleles)

**Design inspiration:** visible combat expression and breeding value can differ. Do not conflate an allele masked by another allele with a property suppressed by the game's capacity budget.

### 4. Breeding reshuffles existing material

During meiosis, crossing over exchanges segments between paired chromosomes, producing new combinations. Nearby loci are more likely to be inherited together: linkage. Inheritance is therefore not simply an independent coin flip for every gene. [NHGRI: Crossing Over](https://www.genome.gov/genetics-glossary/Crossing-Over), [Linkage](https://www.genome.gov/genetics-glossary/Linkage)

**Design inspiration:** inheritable clusters could preserve useful family packages while occasional recombination permits new combinations. This adds complexity; use only if preserving combinations is important to the intended breeding experience.

### 5. Interactions are the strongest fit for synergy

Polygenic traits involve multiple genes. Epistasis means one gene modifies or masks the expression of another. Gene regulation controls when, where, and how much genes are expressed. These are distinct concepts, not interchangeable labels. [NHGRI: Polygenic Trait](https://www.genome.gov/genetics-glossary/Polygenic-Trait), [Epistasis](https://www.genome.gov/genetics-glossary/Epistasis), [Gene Regulation](https://www.genome.gov/genetics-glossary/Gene-Regulation)

**Design inspiration:** combining variants can change function, rather than merely adding bonuses. For example, a fictional secretion mechanism plus a delivery mechanism could yield a ranged secretion attack; a regulatory variant might alter its trigger. The game examples are inventions, not biological claims. Tactical synergy between teammates remains a separate layer from genetic interaction within one creature.

### 6. Mutation is not a guaranteed upgrade

Mutation changes DNA. Whether changes are inherited depends on where they occur; mutations in germline cells can pass to offspring, unlike ordinary somatic changes. Mutation should not be equated with improvement. [NHGRI: Mutation](https://www.genome.gov/genetics-glossary/Mutation)

**Design inspiration:** distinguish inheritance/recombination from introduction of a new variant. Avoid making rare beneficial mutation the sole path to a needed counter.

### Biological limit of the analogy

There is no universal biological 0–100% gene-capacity meter that truncates traits. Keep capacity explicitly fictional. Regulatory mechanisms can inspire how expression works but do not scientifically validate that budget. Likewise, changing an individual creature during its lifetime is not the same thing as population evolution over generations.

## Games: documented mechanics and lessons

| Game | Source-backed approach | Design interpretation, not an empirical outcome |
|---|---|---|
| Axie Infinity | Whitepaper describes six body parts, each carrying dominant, recessive, and minor-recessive entries; dominant entry determines the visible part. Listed offspring-part probabilities per parent's entries are 37.5%, 9.375%, and 3.125%; across both parents these sum to 100%. | Explicit odds and carried-but-unexpressed material are useful precedents. D/R1/R2 is a game-specific weighted hierarchy, not ordinary diploid Mendelian inheritance. Body-part assembly alone does not provide the requested fine-grained generative kit. |
| Niche | Developer describes over 100 genes, dominant/recessive and codominant inheritance, population-genetics mechanics, and survival across five biomes. | Strong inspiration for contextual usefulness and biome specialization. Do not copy extinction or creature-loss stakes into this game's no-defeat-loss contract. Exact gene formulas were not verified. |
| Wobbledogs | Developer describes genetically controlled bodies, mutations during pupal stages, and food-driven gut flora influencing mutations. Body structures affect physical interactions with the world. | Useful precedent for development after birth and meaningful morphology. Its surprising sandbox outcomes are not evidence of predictable competitive breeding. Exact allele formulas were not verified. |

Sources: [Axie whitepaper](https://whitepaper.axieinfinity.com/gameplay/breeding), [Niche developer description](https://store.steampowered.com/app/440650/Niche__a_genetics_survival_game/), [Wobbledogs developer description](https://store.steampowered.com/app/1424330/Wobbledogs/).

Minecraft and ARK were discovery leads only; search results relied on community material, so their detailed formulas are omitted from this primary-source quick study.

## Suggested direction to discuss, not adopt yet

Borrow the causal structure, not molecular detail:

1. Inherited variants form a genotype.
2. Interactions and expression rules produce a combat phenotype.
3. Separate gene capacity constrains that expression through explicitly fictional rules.
4. Breeding recombines variants with visible probabilities; mutation is a separate source of novelty.
5. The interface explains expressed effects, inactive material, and relevant interactions, rather than requiring players to decode raw tokens.

A letter-digit label can identify an allele, but its syntax does not itself determine how intuitive breeding is. Intuition depends on stable causal relationships, meaningful comparisons, and understandable inheritance. Do not select token count, chromosome count, paired copies, or dominance rules before deciding which breeding experience those structures need to support.

Next discussion: should players mainly preserve known functional packages, combine complementary variants into new functions, or balance both? Then compare a small worked inheritance example before committing to a representation. The expression algorithm remains a separate decision ticket.
